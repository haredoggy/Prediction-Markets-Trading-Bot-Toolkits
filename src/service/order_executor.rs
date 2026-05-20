//! High-level execution surface: glue between metadata → eligibility →
//! sizing → exposure → risk → signing → POST → position recording.
//!
//! Every bot routes its decided trades through [`OrderExecutor::execute`].
//! Safety flags are checked here, so individual bots never accidentally
//! bypass `enable_trading` or `mock_trading`.

use std::sync::Arc;

use crate::config::AppConfig;
use crate::models::{OrderType, PlannedOrder, Side, WhaleTrade};
use crate::service::clob::{ClobClient, SignedOrder};
use crate::service::eligibility::{self, Eligibility};
use crate::service::market_cache::{MarketCache, MarketInfo};
use crate::service::position_store::{OpenPosition, PositionStore};
use crate::service::risk_guard::{BlockReason, RiskCheck, RiskGuard};
use crate::service::strategy;
use crate::utils;
use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn};

pub struct OrderExecutor {
    cfg: AppConfig,
    clob: Option<Arc<ClobClient>>,
    risk: Arc<RiskGuard>,
    markets: Arc<MarketCache>,
    positions: Arc<PositionStore>,
}

#[derive(Debug, Clone)]
pub enum ExecutionOutcome {
    Skipped(SkipReason),
    DryRun(SignedOrder),
    Submitted {
        order_id: Option<String>,
        signed: SignedOrder,
    },
}

#[derive(Debug, Clone)]
pub enum SkipReason {
    BelowSizing,
    MarketMetadataUnavailable,
    Ineligible(Eligibility),
    ExposureCategoryCap { category: String, cap: f64, current: f64, want: f64 },
    ExposureTagCap { tag: String, cap: f64, current: f64, want: f64 },
    RiskBlocked(BlockReason),
    TradingDisabled,
    AlreadyOpen,
    /// Whale closed/reduced their position. We delegate exits to the TP/SL
    /// monitor rather than mirroring the unwind, so this is a no-op by design.
    WhaleExitIgnored,
}

impl OrderExecutor {
    pub fn new(
        cfg: AppConfig,
        risk: Arc<RiskGuard>,
        markets: Arc<MarketCache>,
        positions: Arc<PositionStore>,
    ) -> Result<Self> {
        let clob = match ClobClient::new(&cfg) {
            Ok(c) => Some(Arc::new(c)),
            Err(e) if cfg.bot.mock_trading || !cfg.bot.enable_trading => {
                warn!(error = ?e, "CLOB client not initialised; dry-run only");
                None
            }
            Err(e) => return Err(e),
        };
        Ok(Self {
            cfg,
            clob,
            risk,
            markets,
            positions,
        })
    }

    /// Return the underlying CLOB client (for the position monitor's exit path).
    pub fn clob(&self) -> Option<Arc<ClobClient>> {
        self.clob.as_ref().cloned()
    }

    pub fn positions(&self) -> Arc<PositionStore> {
        Arc::clone(&self.positions)
    }

    pub async fn execute(&self, trade: &WhaleTrade) -> Result<ExecutionOutcome> {
        // 0a. Whale sells are not mirrored — TP/SL is the configured exit path.
        if trade.side == Side::Sell {
            return Ok(ExecutionOutcome::Skipped(SkipReason::WhaleExitIgnored));
        }
        // 0b. Don't pyramid into a market we're already long.
        if self.positions.get(&trade.token_id).is_some() {
            return Ok(ExecutionOutcome::Skipped(SkipReason::AlreadyOpen));
        }

        // 1. Resolve market metadata for category/tags/closed lookup.
        let market = match self.markets.by_token_id(&trade.token_id).await {
            Ok(m) => m,
            Err(e) => {
                warn!(token = %trade.token_id, error = ?e, "market metadata lookup failed");
                return Ok(ExecutionOutcome::Skipped(
                    SkipReason::MarketMetadataUnavailable,
                ));
            }
        };

        // 2. Eligibility (allowlist / blocklist / closed).
        let eligibility = eligibility::check(&self.cfg.filters, &market);
        if eligibility != Eligibility::Allowed {
            info!(
                slug = %market.slug,
                category = ?market.category,
                ?eligibility,
                "market not eligible"
            );
            return Ok(ExecutionOutcome::Skipped(SkipReason::Ineligible(
                eligibility,
            )));
        }

        // 3. Sizing.
        let sizing = strategy::size_for_trade(&self.cfg.strategy, trade);
        if let Some(r) = sizing.skipped {
            info!(?r, "sizing skipped trade");
            return Ok(ExecutionOutcome::Skipped(SkipReason::BelowSizing));
        }

        // 4. Per-category / per-tag exposure caps (entries only).
        if trade.side == Side::Buy {
            if let Some(skip) = self.check_exposure(&market, sizing.copy_usd) {
                info!(?skip, "exposure cap blocked entry");
                return Ok(ExecutionOutcome::Skipped(skip));
            }
        }

        // 5. Risk guard.
        match self.risk.check_fast(trade) {
            RiskCheck::Allow => {}
            RiskCheck::FetchBook => {
                // Caller (or a follow-up) should pull the book and call
                // `risk.check_with_book`. For now we proceed but flag the
                // pre-trade book fetch as the next integration point.
            }
            RiskCheck::Block(reason) => {
                warn!(?reason, "risk guard blocked trade");
                return Ok(ExecutionOutcome::Skipped(SkipReason::RiskBlocked(reason)));
            }
        }

        // 6. Build the order.
        let limit_price = limit_price_for(trade, &self.cfg);
        let shares = utils::usd_to_shares(sizing.copy_usd, limit_price);
        let order_type = order_type_for(trade.side);
        let planned = PlannedOrder {
            venue: trade.venue,
            token_id: trade.token_id.clone(),
            side: trade.side,
            shares,
            limit_price,
            usd_notional: sizing.copy_usd,
            order_type,
            source_trade_hash: trade.tx_hash.clone(),
        };

        let Some(clob) = self.clob.as_ref() else {
            return Ok(ExecutionOutcome::Skipped(SkipReason::TradingDisabled));
        };

        // 7. Sign.
        let signed = clob
            .build_signed_order(&planned, order_type, self.cfg.trading.order_expiration_secs)
            .await?;

        // 8. Dispatch (or dry-run).
        if !self.cfg.live_trading_allowed() {
            info!(
                token = %planned.token_id,
                side = ?planned.side,
                shares = planned.shares,
                price = planned.limit_price,
                "dry-run: signed order built but not submitted"
            );
            // Even in dry-run, record the position so the monitor exercises
            // the TP/SL path against real midprices.
            self.record_open(&market, &planned);
            return Ok(ExecutionOutcome::DryRun(signed));
        }

        let resp = clob.post_order(signed.clone(), order_type).await?;
        self.record_open(&market, &planned);
        Ok(ExecutionOutcome::Submitted {
            order_id: resp.order_id,
            signed,
        })
    }

    fn check_exposure(&self, market: &MarketInfo, want_usd: f64) -> Option<SkipReason> {
        let filters = &self.cfg.filters;
        if let Some(cat) = market.category.as_ref() {
            if let Some(&cap) = filters
                .per_category_max_open_usd
                .iter()
                .find(|(k, _)| ci_eq(k, cat))
                .map(|(_, v)| v)
            {
                let current = self.positions.open_usd_by_category(cat);
                if current + want_usd > cap {
                    return Some(SkipReason::ExposureCategoryCap {
                        category: cat.clone(),
                        cap,
                        current,
                        want: want_usd,
                    });
                }
            }
        }
        for tag in &market.tags {
            if let Some(&cap) = filters
                .per_tag_max_open_usd
                .iter()
                .find(|(k, _)| ci_eq(k, tag))
                .map(|(_, v)| v)
            {
                let current = self.positions.open_usd_by_tag(tag);
                if current + want_usd > cap {
                    return Some(SkipReason::ExposureTagCap {
                        tag: tag.clone(),
                        cap,
                        current,
                        want: want_usd,
                    });
                }
            }
        }
        None
    }

    fn record_open(&self, market: &MarketInfo, planned: &PlannedOrder) {
        // Only entries open positions; exits remove them in the monitor.
        if planned.side != Side::Buy {
            return;
        }
        let (tp_pct, sl_pct) = self.tp_sl_for(market);
        let pos = OpenPosition {
            token_id: planned.token_id.clone(),
            slug: market.slug.clone(),
            category: market.category.clone(),
            tags: market.tags.clone(),
            side: planned.side,
            entry_price: planned.limit_price,
            shares: planned.shares,
            usd_notional: planned.usd_notional,
            take_profit_pct: tp_pct,
            stop_loss_pct: sl_pct,
            opened_at: Utc::now(),
        };
        self.positions.open(pos);
    }

    fn tp_sl_for(&self, market: &MarketInfo) -> (f64, f64) {
        let tp_default = self.cfg.tp_sl.default_take_profit_pct;
        let sl_default = self.cfg.tp_sl.default_stop_loss_pct;
        if let Some(cat) = market.category.as_ref() {
            let tp = self
                .cfg
                .tp_sl
                .per_category_tp_pct
                .iter()
                .find(|(k, _)| ci_eq(k, cat))
                .map(|(_, v)| *v)
                .unwrap_or(tp_default);
            let sl = self
                .cfg
                .tp_sl
                .per_category_sl_pct
                .iter()
                .find(|(k, _)| ci_eq(k, cat))
                .map(|(_, v)| *v)
                .unwrap_or(sl_default);
            (tp, sl)
        } else {
            (tp_default, sl_default)
        }
    }
}

fn limit_price_for(trade: &WhaleTrade, cfg: &AppConfig) -> f64 {
    let buf = cfg.trading.price_buffer;
    let raw = match trade.side {
        Side::Buy => trade.price + buf,
        Side::Sell => trade.price - buf,
    };
    utils::clamp_price(raw)
}

fn order_type_for(side: Side) -> OrderType {
    match side {
        Side::Buy => OrderType::Fak,
        Side::Sell => OrderType::Gtd,
    }
}

fn ci_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.chars().zip(b.chars()).all(|(x, y)| {
        x.to_ascii_lowercase() == y.to_ascii_lowercase()
    })
}

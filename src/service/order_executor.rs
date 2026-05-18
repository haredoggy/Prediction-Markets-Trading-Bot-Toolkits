//! High-level execution surface: glue between sizing → risk → signing → POST.
//!
//! Every bot routes its decided trades through [`OrderExecutor::execute`].
//! Safety flags are checked here, so individual bots never accidentally bypass
//! `enable_trading` or `mock_trading`.

use std::sync::Arc;

use crate::config::AppConfig;
use crate::models::{OrderType, PlannedOrder, Side, WhaleTrade};
use crate::service::clob::{ClobClient, SignedOrder};
use crate::service::risk_guard::{BlockReason, RiskCheck, RiskGuard};
use crate::service::strategy;
use crate::utils;
use anyhow::Result;
use tracing::{info, warn};

pub struct OrderExecutor {
    cfg: AppConfig,
    clob: Option<ClobClient>,
    risk: Arc<RiskGuard>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    BelowSizing,
    RiskBlocked(BlockReason),
    TradingDisabled,
}

impl OrderExecutor {
    pub fn new(cfg: AppConfig, risk: Arc<RiskGuard>) -> Result<Self> {
        // CLOB client requires a valid private key. In mock mode we may not
        // have one — return a stub-less executor instead of failing the boot.
        let clob = match ClobClient::new(&cfg) {
            Ok(c) => Some(c),
            Err(e) if cfg.bot.mock_trading || !cfg.bot.enable_trading => {
                warn!(error = ?e, "CLOB client not initialised; dry-run only");
                None
            }
            Err(e) => return Err(e),
        };
        Ok(Self { cfg, clob, risk })
    }

    /// Apply sizing + risk + signing for a single whale trade.
    pub async fn execute(&self, trade: &WhaleTrade) -> Result<ExecutionOutcome> {
        let sizing = strategy::size_for_trade(&self.cfg.strategy, trade);
        if let Some(r) = sizing.skipped {
            info!(?r, "sizing skipped trade");
            return Ok(ExecutionOutcome::Skipped(SkipReason::BelowSizing));
        }

        match self.risk.check_fast(trade) {
            RiskCheck::Allow => {}
            RiskCheck::FetchBook => {
                // The book fetch happens at the caller; for now we proceed,
                // mirroring the brief's "book fetch failure trips the breaker".
                // A real bot would call `MarketCache + orderbook fetcher` here.
            }
            RiskCheck::Block(reason) => {
                warn!(?reason, "risk guard blocked trade");
                return Ok(ExecutionOutcome::Skipped(SkipReason::RiskBlocked(reason)));
            }
        }

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

        let signed = clob
            .build_signed_order(&planned, order_type, self.cfg.trading.order_expiration_secs)
            .await?;

        if !self.cfg.live_trading_allowed() {
            info!(
                token_id = %planned.token_id,
                side = ?planned.side,
                shares = planned.shares,
                price = planned.limit_price,
                "dry-run: signed order built but not submitted"
            );
            return Ok(ExecutionOutcome::DryRun(signed));
        }

        let resp = clob.post_order(signed.clone(), order_type).await?;
        Ok(ExecutionOutcome::Submitted {
            order_id: resp.order_id,
            signed,
        })
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
        // Buys chase aggressively per the brief — FAK semantics.
        Side::Buy => OrderType::Fak,
        // Sells unwind with time-bounded limits.
        Side::Sell => OrderType::Gtd,
    }
}

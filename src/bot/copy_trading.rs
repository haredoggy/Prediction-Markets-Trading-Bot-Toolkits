//! Copy-trading bot — production-ready.
//!
//! Architecture (matches §2 of the technical brief):
//!
//! 1. **Ingestion**: Polygon WebSocket `eth_subscribe`/`logs` filtered to the
//!    configured CTF Exchange contracts, topic-0 = `OrderFilled`, and the
//!    watched whale address packed as topic-2 (maker). Server-side filtering
//!    means the bot is woken only when the whale actually transacts.
//! 2. **Parse**: decode raw logs into [`WhaleTrade`] via [`service::parse`].
//! 3. **Eligibility**: resolve market metadata via Gamma, then check against
//!    the operator's allow/block lists ([`service::eligibility`]).
//! 4. **Sizing**: apply the configured copy strategy ([`service::strategy`]).
//! 5. **Exposure caps**: per-category and per-tag open-USD limits enforced
//!    via [`service::position_store`].
//! 6. **Risk**: fast in-memory check, optional book/depth check
//!    ([`service::risk_guard`]).
//! 7. **Execute**: build EIP-712 signed CTF order, post via L2 auth
//!    ([`service::clob`], [`service::order_executor`]).
//! 8. **TP/SL**: background monitor polls midprice for every open position
//!    and submits an exit FAK when P&L crosses the configured thresholds
//!    ([`service::position_monitor`]).
//!
//! Safety: `enable_trading=false` OR `mock_trading=true` keeps the executor
//! in dry-run mode — the full pipeline runs but signed orders are logged,
//! not submitted. Dry-run also records positions locally so the TP/SL
//! monitor exercises against real midprices.

use crate::config::AppConfig;
use crate::service::{
    clob::ClobClient,
    market_cache::MarketCache,
    midprice::{ClobMidpriceSource, MidpriceSource},
    onchain::{spawn_subscription, LogFilter, RawLog},
    order_executor::{ExecutionOutcome, OrderExecutor},
    parse::{decode_whale_trade, order_filled_topic},
    position_monitor,
    position_store::PositionStore,
    risk_guard::RiskGuard,
};
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

const LOG_CHANNEL_CAPACITY: usize = 256;

pub async fn run(cfg: AppConfig) -> Result<()> {
    let whale = cfg
        .bot
        .wallets_to_track
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("config.bot.wallets_to_track is empty"))?;

    info!(
        whale = %whale,
        enable_trading = cfg.bot.enable_trading,
        mock_trading = cfg.bot.mock_trading,
        strict_allowlist = cfg.filters.is_strict(),
        tp_sl_enabled = cfg.tp_sl.enabled,
        "starting copy-trading bot"
    );

    let http = Client::builder()
        .user_agent("polymarket-toolkits/0.1")
        .build()?;
    let risk = RiskGuard::new(cfg.risk.clone());
    let markets = MarketCache::new(http.clone(), cfg.site.gamma_api_base.clone());
    let positions = PositionStore::new();

    let executor = OrderExecutor::new(
        cfg.clone(),
        Arc::clone(&risk),
        Arc::clone(&markets),
        Arc::clone(&positions),
    )?;

    // TP/SL monitor — only spawn if we have a CLOB client (cfg permitted it).
    if cfg.tp_sl.enabled {
        if let Some(clob) = executor.clob() {
            let midprice: Arc<dyn MidpriceSource> = Arc::new(ClobMidpriceSource::new(
                http.clone(),
                cfg.site.clob_api_base.clone(),
            ));
            let live = cfg.live_trading_allowed();
            position_monitor::spawn(
                cfg.tp_sl.clone(),
                Arc::clone(&positions),
                clob,
                midprice,
                live,
                cfg.trading.price_buffer,
                cfg.trading.order_expiration_secs,
            );
            info!(
                poll_interval_secs = cfg.tp_sl.poll_interval_secs,
                "TP/SL monitor spawned"
            );
        } else {
            warn!(
                "TP/SL enabled but no CLOB client (missing credentials?) — \
                 entries will still record positions, but exits won't fire"
            );
        }
    }

    let filter = build_filter(&cfg, &whale)?;
    let (tx, mut rx) = mpsc::channel::<RawLog>(LOG_CHANNEL_CAPACITY);
    let _sub = spawn_subscription(cfg.site.polygon_ws_url.clone(), filter, tx);

    let mut shutdown = Box::pin(tokio::signal::ctrl_c());

    loop {
        tokio::select! {
            biased;
            _ = &mut shutdown => {
                info!(open_positions = positions.len(), "shutdown signal received");
                return Ok(());
            }
            maybe_log = rx.recv() => {
                let Some(log) = maybe_log else {
                    warn!("on-chain subscription channel closed");
                    return Ok(());
                };
                if let Err(e) = handle_log(&executor, &whale, &log).await {
                    error!(error = ?e, tx = %log.tx_hash, "handle_log failed");
                }
            }
        }
    }
}

async fn handle_log(executor: &OrderExecutor, whale: &str, log: &RawLog) -> Result<()> {
    let Some(trade) = decode_whale_trade(log, whale)? else {
        return Ok(());
    };
    info!(
        token = %trade.token_id,
        side = ?trade.side,
        shares = trade.shares,
        usd = trade.usd_notional,
        tx = %log.tx_hash,
        "whale trade detected"
    );
    match executor.execute(&trade).await? {
        ExecutionOutcome::Skipped(r) => info!(?r, "execution skipped"),
        ExecutionOutcome::DryRun(signed) => info!(
            token = %signed.token_id,
            shares = %signed.taker_amount,
            "dry-run order signed (not submitted)"
        ),
        ExecutionOutcome::Submitted { order_id, .. } => info!(
            order_id = ?order_id,
            "order submitted to Polymarket CLOB"
        ),
    }
    Ok(())
}

fn build_filter(cfg: &AppConfig, whale: &str) -> Result<LogFilter> {
    let exchanges = vec![
        cfg.exchange.ctf_exchange_address.to_lowercase(),
        cfg.exchange.neg_risk_exchange_address.to_lowercase(),
    ];
    let topic0 = format!("0x{}", hex::encode(order_filled_topic().as_slice()));
    let whale_topic = pad_address_to_topic(whale)?;
    Ok(LogFilter {
        address: exchanges,
        topics: vec![
            Some(vec![topic0]),
            None,
            Some(vec![whale_topic]),
        ],
    })
}

fn pad_address_to_topic(addr: &str) -> Result<String> {
    let trimmed = addr.trim().trim_start_matches("0x").to_lowercase();
    if trimmed.len() != 40 {
        return Err(anyhow!("address must be 20 bytes / 40 hex chars"));
    }
    Ok(format!("0x{}{}", "0".repeat(24), trimmed))
}

//! Take-profit / stop-loss monitor.
//!
//! Periodically polls the midprice of every open position. When the P&L
//! relative to the recorded entry crosses the take-profit or stop-loss
//! threshold, the monitor synthesises an exit `PlannedOrder` and posts it
//! through the existing executor — same signing path, same risk gates.

use crate::config::TpSlConfig;
use crate::models::{OrderType, PlannedOrder, Side, VenueId};
use crate::service::clob::ClobClient;
use crate::service::midprice::MidpriceSource;
use crate::service::position_store::{OpenPosition, PositionStore};
use crate::utils;
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    TakeProfit,
    StopLoss,
}

pub fn check_exit(pos: &OpenPosition, midprice: f64) -> Option<ExitReason> {
    let pnl = pos.pnl_pct(midprice);
    if pnl >= pos.take_profit_pct {
        Some(ExitReason::TakeProfit)
    } else if pnl <= -pos.stop_loss_pct {
        Some(ExitReason::StopLoss)
    } else {
        None
    }
}

/// Spawns the monitor task. The handle lets the caller observe shutdown
/// independently of the main bot loop.
pub fn spawn(
    cfg: TpSlConfig,
    positions: Arc<PositionStore>,
    clob: Arc<ClobClient>,
    midprice: Arc<dyn MidpriceSource>,
    live_trading: bool,
    price_buffer: f64,
    order_expiration_secs: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if !cfg.enabled {
            info!("TP/SL monitor disabled in config — exiting task");
            return;
        }
        let mut ticker = interval(Duration::from_secs(cfg.poll_interval_secs.max(1)));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            let snapshot = positions.snapshot();
            for pos in snapshot {
                if let Err(e) = monitor_once(
                    &pos,
                    &positions,
                    clob.as_ref(),
                    midprice.as_ref(),
                    live_trading,
                    price_buffer,
                    order_expiration_secs,
                )
                .await
                {
                    error!(
                        token = %pos.token_id,
                        error = ?e,
                        "tp/sl tick failed for position"
                    );
                }
            }
        }
    })
}

async fn monitor_once(
    pos: &OpenPosition,
    positions: &PositionStore,
    clob: &ClobClient,
    midprice: &dyn MidpriceSource,
    live_trading: bool,
    price_buffer: f64,
    order_expiration_secs: u64,
) -> Result<()> {
    let mp = midprice.midprice(&pos.token_id).await?;
    let pnl = pos.pnl_pct(mp);
    debug!(
        token = %pos.token_id,
        mid = mp,
        entry = pos.entry_price,
        pnl_pct = pnl,
        "tp/sl tick"
    );

    let Some(reason) = check_exit(pos, mp) else {
        return Ok(());
    };

    info!(
        token = %pos.token_id,
        slug = %pos.slug,
        ?reason,
        pnl_pct = pnl,
        midprice = mp,
        "TP/SL triggered — submitting exit"
    );

    let exit_side = pos.side.flip();
    let limit_price = match exit_side {
        // Selling out — accept a buffer below the mid to make sure we fill.
        Side::Sell => utils::clamp_price(mp - price_buffer),
        // Buying to cover — pay up by the buffer.
        Side::Buy => utils::clamp_price(mp + price_buffer),
    };

    let planned = PlannedOrder {
        venue: VenueId::Polymarket,
        token_id: pos.token_id.clone(),
        side: exit_side,
        shares: pos.shares,
        limit_price,
        usd_notional: pos.shares * limit_price,
        order_type: OrderType::Fak,
        source_trade_hash: None,
    };

    let signed = clob
        .build_signed_order(&planned, OrderType::Fak, order_expiration_secs)
        .await?;

    if live_trading {
        match clob.post_order(signed.clone(), OrderType::Fak).await {
            Ok(resp) => {
                info!(
                    order_id = ?resp.order_id,
                    token = %pos.token_id,
                    "exit order submitted"
                );
                positions.close(&pos.token_id);
            }
            Err(e) => {
                warn!(error = ?e, token = %pos.token_id, "exit POST failed; will retry next tick");
            }
        }
    } else {
        info!(
            token = %pos.token_id,
            shares = planned.shares,
            limit = planned.limit_price,
            "dry-run exit signed (not submitted) — closing position locally"
        );
        positions.close(&pos.token_id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn pos(entry: f64, tp: f64, sl: f64, side: Side) -> OpenPosition {
        OpenPosition {
            token_id: "t".into(),
            slug: "s".into(),
            category: None,
            tags: vec![],
            side,
            entry_price: entry,
            shares: 100.0,
            usd_notional: 100.0 * entry,
            take_profit_pct: tp,
            stop_loss_pct: sl,
            opened_at: Utc::now(),
        }
    }

    #[test]
    fn take_profit_triggers() {
        let p = pos(0.50, 30.0, 20.0, Side::Buy);
        assert_eq!(check_exit(&p, 0.70), Some(ExitReason::TakeProfit));
    }

    #[test]
    fn stop_loss_triggers() {
        let p = pos(0.50, 30.0, 20.0, Side::Buy);
        assert_eq!(check_exit(&p, 0.39), Some(ExitReason::StopLoss));
    }

    #[test]
    fn flat_range_does_not_trigger() {
        let p = pos(0.50, 30.0, 20.0, Side::Buy);
        assert_eq!(check_exit(&p, 0.55), None);
    }

    #[test]
    fn sell_inverted_pnl_logic() {
        let p = pos(0.50, 30.0, 20.0, Side::Sell);
        // Price went DOWN — short position is in profit.
        assert_eq!(check_exit(&p, 0.30), Some(ExitReason::TakeProfit));
        // Price went UP — short position hit stop.
        assert_eq!(check_exit(&p, 0.65), Some(ExitReason::StopLoss));
    }
}

//! Circuit breaker + orderbook depth guard.
//!
//! Two layers of protection on the hot path:
//! 1. **Fast check** — counts consecutive large trades per outcome token
//!    within a rolling window. Trips synchronously without any I/O.
//! 2. **Slow check** — when the fast check requests it, fetches the orderbook
//!    and validates that depth on the relevant side meets `min_depth_usd`.
//!
//! Both checks honour `trip_duration_secs`: once the breaker fires, every
//! subsequent decision is blocked until the cooldown elapses.

use crate::config::RiskConfig;
use crate::models::{BookSnapshot, Side, WhaleTrade};
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskCheck {
    Allow,
    /// Require a book fetch + `check_with_book` before allowing.
    FetchBook,
    Block(BlockReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockReason {
    Tripped,
    ConsecutiveLargeTrades,
    InsufficientDepth,
}

#[derive(Debug)]
pub struct RiskGuard {
    cfg: RiskConfig,
    inner: Mutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    /// token_id → ring buffer of "large trade" timestamps.
    history: HashMap<String, VecDeque<Instant>>,
    tripped_until: Option<Instant>,
}

impl RiskGuard {
    pub fn new(cfg: RiskConfig) -> Arc<Self> {
        Arc::new(Self {
            cfg,
            inner: Mutex::new(Inner {
                history: HashMap::new(),
                tripped_until: None,
            }),
        })
    }

    /// Synchronous, in-memory only. Call before any network I/O.
    pub fn check_fast(&self, trade: &WhaleTrade) -> RiskCheck {
        let now = Instant::now();
        let mut g = self.inner.lock();

        if let Some(until) = g.tripped_until {
            if now < until {
                return RiskCheck::Block(BlockReason::Tripped);
            }
            g.tripped_until = None;
        }

        if trade.shares < self.cfg.large_trade_shares {
            return RiskCheck::Allow;
        }

        let window = Duration::from_secs(self.cfg.sequence_window_secs);
        let bucket = g.history.entry(trade.token_id.clone()).or_default();

        // Drop expired entries from the head.
        while let Some(&front) = bucket.front() {
            if now.duration_since(front) > window {
                bucket.pop_front();
            } else {
                break;
            }
        }
        bucket.push_back(now);

        if bucket.len() as u32 >= self.cfg.consecutive_trigger {
            g.tripped_until =
                Some(now + Duration::from_secs(self.cfg.trip_duration_secs));
            return RiskCheck::Block(BlockReason::ConsecutiveLargeTrades);
        }

        // Large trade — pulse the book check before allowing.
        RiskCheck::FetchBook
    }

    /// Phase two, after the caller has fetched the book.
    pub fn check_with_book(&self, book: &BookSnapshot, side: Side) -> RiskCheck {
        let depth = book.depth_usd(side, 5);
        if depth < self.cfg.min_depth_usd {
            self.trip(BlockReason::InsufficientDepth);
            RiskCheck::Block(BlockReason::InsufficientDepth)
        } else {
            RiskCheck::Allow
        }
    }

    /// Manual trip — e.g. when a book fetch fails, prefer tripping
    /// to firing blind.
    pub fn trip(&self, _reason: BlockReason) {
        let mut g = self.inner.lock();
        g.tripped_until =
            Some(Instant::now() + Duration::from_secs(self.cfg.trip_duration_secs));
    }

    pub fn is_tripped(&self) -> bool {
        let g = self.inner.lock();
        g.tripped_until
            .map(|t| Instant::now() < t)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Side, VenueId};

    fn cfg() -> RiskConfig {
        RiskConfig {
            large_trade_shares: 1000.0,
            consecutive_trigger: 2,
            sequence_window_secs: 30,
            min_depth_usd: 200.0,
            trip_duration_secs: 60,
        }
    }

    fn trade(shares: f64) -> WhaleTrade {
        WhaleTrade {
            venue: VenueId::Polymarket,
            maker: "0x".into(),
            side: Side::Buy,
            token_id: "tok".into(),
            shares,
            price: 0.5,
            usd_notional: shares * 0.5,
            tx_hash: None,
            block_number: None,
            observed_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn small_trades_always_pass() {
        let g = RiskGuard::new(cfg());
        assert_eq!(g.check_fast(&trade(10.0)), RiskCheck::Allow);
        assert_eq!(g.check_fast(&trade(10.0)), RiskCheck::Allow);
    }

    #[test]
    fn first_large_requests_book() {
        let g = RiskGuard::new(cfg());
        assert_eq!(g.check_fast(&trade(2000.0)), RiskCheck::FetchBook);
    }

    #[test]
    fn second_consecutive_large_trips() {
        let g = RiskGuard::new(cfg());
        let _ = g.check_fast(&trade(2000.0));
        let r = g.check_fast(&trade(2000.0));
        matches!(r, RiskCheck::Block(BlockReason::ConsecutiveLargeTrades));
        assert!(g.is_tripped());
    }

    #[test]
    fn depth_check_blocks_thin_book() {
        let g = RiskGuard::new(cfg());
        let book = BookSnapshot {
            bids: vec![],
            asks: vec![crate::models::BookLevel { price: 0.5, size: 10.0 }],
        };
        let r = g.check_with_book(&book, Side::Buy);
        assert_eq!(r, RiskCheck::Block(BlockReason::InsufficientDepth));
    }
}

//! Copy-sizing strategies.
//!
//! Implements the three strategies described in the technical brief:
//! `PERCENTAGE`, `FIXED`, and `ADAPTIVE`. The adaptive curve interpolates
//! linearly in log-space so small whale legs lean toward `adaptive_max_percent`
//! and large legs decay smoothly toward `adaptive_min_percent`.

use crate::config::{CopyStrategy, StrategyConfig};
use crate::models::WhaleTrade;

/// Outcome of running a sizing strategy on one observed whale trade.
#[derive(Debug, Clone)]
pub struct SizingDecision {
    pub copy_usd: f64,
    pub effective_percent: f64,
    pub skipped: Option<SkipReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    BelowMinSharesToCopy,
    BelowMinOrderSize,
    NonPositiveNotional,
}

pub fn size_for_trade(cfg: &StrategyConfig, trade: &WhaleTrade) -> SizingDecision {
    if trade.shares < cfg.min_whale_shares_to_copy {
        return SizingDecision {
            copy_usd: 0.0,
            effective_percent: 0.0,
            skipped: Some(SkipReason::BelowMinSharesToCopy),
        };
    }
    if trade.usd_notional <= 0.0 {
        return SizingDecision {
            copy_usd: 0.0,
            effective_percent: 0.0,
            skipped: Some(SkipReason::NonPositiveNotional),
        };
    }

    let (base_usd, effective_percent) = match cfg.copy_strategy {
        CopyStrategy::Percentage => {
            let pct = cfg.copy_size;
            (trade.usd_notional * pct / 100.0, pct)
        }
        CopyStrategy::Fixed => (cfg.copy_size, 0.0),
        CopyStrategy::Adaptive => {
            let pct = adaptive_percent(cfg, trade.usd_notional);
            (trade.usd_notional * pct / 100.0, pct)
        }
    };

    let after_multiplier = base_usd * cfg.trade_multiplier;
    let capped = after_multiplier.min(cfg.max_order_size_usd);

    if capped < cfg.min_order_size_usd {
        return SizingDecision {
            copy_usd: capped,
            effective_percent,
            skipped: Some(SkipReason::BelowMinOrderSize),
        };
    }

    SizingDecision {
        copy_usd: capped,
        effective_percent,
        skipped: None,
    }
}

/// Smooth interpolation: pct goes from `adaptive_max_percent` at zero notional
/// down to `adaptive_min_percent` as notional grows past `adaptive_threshold_usd`.
fn adaptive_percent(cfg: &StrategyConfig, notional_usd: f64) -> f64 {
    if cfg.adaptive_threshold_usd <= 0.0 {
        return cfg.adaptive_min_percent;
    }
    let ratio = notional_usd / cfg.adaptive_threshold_usd;
    // Exponential decay toward the floor; ratio=1 → halfway between min and max.
    let weight = (-ratio.ln_1p()).exp();
    cfg.adaptive_min_percent
        + (cfg.adaptive_max_percent - cfg.adaptive_min_percent) * weight
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Side, VenueId};

    fn cfg(strategy: CopyStrategy) -> StrategyConfig {
        StrategyConfig {
            copy_strategy: strategy,
            copy_size: 20.0,
            trade_multiplier: 1.0,
            min_order_size_usd: 5.0,
            max_order_size_usd: 500.0,
            min_whale_shares_to_copy: 10.0,
            adaptive_threshold_usd: 1000.0,
            adaptive_min_percent: 5.0,
            adaptive_max_percent: 30.0,
        }
    }

    fn trade(notional: f64, shares: f64) -> WhaleTrade {
        WhaleTrade {
            venue: VenueId::Polymarket,
            maker: "0xtest".into(),
            side: Side::Buy,
            token_id: "1".into(),
            shares,
            price: 0.5,
            usd_notional: notional,
            tx_hash: None,
            block_number: None,
            observed_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn percentage_basic() {
        let d = size_for_trade(&cfg(CopyStrategy::Percentage), &trade(100.0, 200.0));
        assert!((d.copy_usd - 20.0).abs() < 1e-9);
        assert!(d.skipped.is_none());
    }

    #[test]
    fn fixed_uses_copy_size() {
        let d = size_for_trade(&cfg(CopyStrategy::Fixed), &trade(100.0, 200.0));
        assert!((d.copy_usd - 20.0).abs() < 1e-9);
    }

    #[test]
    fn caps_at_max() {
        let mut c = cfg(CopyStrategy::Percentage);
        c.copy_size = 100.0;
        let d = size_for_trade(&c, &trade(10_000.0, 20_000.0));
        assert!((d.copy_usd - c.max_order_size_usd).abs() < 1e-9);
    }

    #[test]
    fn skips_tiny_whale() {
        let d = size_for_trade(&cfg(CopyStrategy::Percentage), &trade(100.0, 1.0));
        assert_eq!(d.skipped, Some(SkipReason::BelowMinSharesToCopy));
    }

    #[test]
    fn adaptive_decays_with_size() {
        let c = cfg(CopyStrategy::Adaptive);
        let small = adaptive_percent(&c, 100.0);
        let big = adaptive_percent(&c, 10_000.0);
        assert!(small > big);
        assert!(big >= c.adaptive_min_percent - 1e-9);
        assert!(small <= c.adaptive_max_percent + 1e-9);
    }
}

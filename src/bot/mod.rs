//! Bot dispatch.
//!
//! Each bot has its own file in this directory. Only [`copy_trading`] is wired
//! end-to-end. The rest expose `run(cfg)` stubs that print a friendly "in
//! development" notice so the binary stays useful to operators experimenting
//! with the engine.

pub mod arbitrage;
pub mod copy_trading;
pub mod cross_market_arb;
pub mod direction_hunting;
pub mod market_maker;
pub mod orderbook_imbalance;
pub mod resolution_sniper;
pub mod spread_farming;
pub mod sports_execution;
pub mod whale_signal;

use crate::config::AppConfig;
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum BotKind {
    CopyTrading,
    BtcArb,
    CrossArb,
    DirectionHunting,
    SpreadFarming,
    Sports,
    ResolutionSniper,
    OrderbookImbalance,
    MarketMaking,
    WhaleSignal,
}

impl BotKind {
    pub fn label(self) -> &'static str {
        match self {
            BotKind::CopyTrading => "Copy Trading",
            BotKind::BtcArb => "BTC 5m / 15m / 1hr Arbitrage",
            BotKind::CrossArb => "Polymarket ↔ Kalshi Cross-Venue Arb",
            BotKind::DirectionHunting => "Direction Hunting",
            BotKind::SpreadFarming => "Spread Farming",
            BotKind::Sports => "Sports Betting Execution",
            BotKind::ResolutionSniper => "Resolution Sniper",
            BotKind::OrderbookImbalance => "Orderbook Imbalance",
            BotKind::MarketMaking => "Market Making",
            BotKind::WhaleSignal => "On-Chain Whale Signal",
        }
    }

    pub fn is_production(self) -> bool {
        matches!(self, BotKind::CopyTrading)
    }
}

pub async fn run(kind: BotKind, cfg: AppConfig) -> Result<()> {
    match kind {
        BotKind::CopyTrading => copy_trading::run(cfg).await,
        BotKind::BtcArb => arbitrage::run(cfg).await,
        BotKind::CrossArb => cross_market_arb::run(cfg).await,
        BotKind::DirectionHunting => direction_hunting::run(cfg).await,
        BotKind::SpreadFarming => spread_farming::run(cfg).await,
        BotKind::Sports => sports_execution::run(cfg).await,
        BotKind::ResolutionSniper => resolution_sniper::run(cfg).await,
        BotKind::OrderbookImbalance => orderbook_imbalance::run(cfg).await,
        BotKind::MarketMaking => market_maker::run(cfg).await,
        BotKind::WhaleSignal => whale_signal::run(cfg).await,
    }
}

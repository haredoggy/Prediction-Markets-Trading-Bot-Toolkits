// ============================================================================
// Bot module — all trading strategy implementations
// ============================================================================
//
// Status legend:
//   ✅  Production-ready
//   🚧  In development (stub compiles, strategy not yet implemented)

/// ✅ Copy Trading Bot — mirrors positions from tracked wallets via positions API
pub mod copy_trading;

/// 🚧 BTC Up/Down Arbitrage — low-latency plays on 5m/15m/1hr BTC markets
pub mod arbitrage;

/// 🚧 Cross-Market Arbitrage — hedged legs across Polymarket ↔ Kalshi
pub mod cross_market_arb;

/// 🚧 Direction Hunting — momentum/flow scanner with TP + SL exit management
pub mod direction_hunting;

/// 🚧 Spread Farming — systematic bid-ask spread capture with P&L logging
pub mod spread_farming;

/// 🚧 Sports Betting Execution — live-sports interface with click-to-FAK execution
pub mod sports_execution;

/// 🚧 Resolution Sniper — buys near-certainty markets before the $1.00 payout
pub mod resolution_sniper;

/// 🚧 Orderbook Imbalance — fades into heavy bid/ask imbalances
pub mod orderbook_imbalance;

/// 🚧 Market Making — two-sided GTD quoting with inventory management
pub mod market_maker;

/// 🚧 On-Chain Whale Signal — detects CLOB interactions from tracked wallets on-chain
pub mod whale_signal;

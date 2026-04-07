/// Kalshi venue adapter.
///
/// Wraps Kalshi's REST API for market data and order execution.
/// Used by the Cross-Market Arbitrage Bot to hedge legs on Kalshi.
pub mod client;

pub use client::KalshiClient;

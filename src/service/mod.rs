/// Polymarket CLOB client construction and authentication
pub mod client;

/// Market metadata cache (slug → token ID resolution)
pub mod market_cache;

/// Order placement and FAK/GTD execution
pub mod orders;

/// Position polling and change detection
pub mod positions;

/// Token ID helpers
pub mod token;

/// CopyTrader execution engine
pub mod trader;

/// Cross-venue real-time price aggregation
pub mod price_feed;

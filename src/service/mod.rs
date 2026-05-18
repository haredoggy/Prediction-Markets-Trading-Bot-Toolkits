//! Shared services consumed by every bot:
//!
//! - [`strategy`]   — copy-sizing strategies (percentage / fixed / adaptive)
//! - [`risk_guard`] — circuit breaker + depth check
//! - [`market_cache`] — slug → CLOB token-id resolution
//! - [`onchain`]    — Polygon WebSocket subscription
//! - [`parse`]      — ABI log decoding for Polymarket exchange events
//! - [`clob`]       — Polymarket CLOB v2 client: EIP-712 signing, order POST
//! - [`order_executor`] — applies sizing, risk, and dispatches to the CLOB

pub mod clob;
pub mod market_cache;
pub mod onchain;
pub mod order_executor;
pub mod parse;
pub mod risk_guard;
pub mod strategy;

/// Polymarket venue adapter.
///
/// Re-exports the existing service layer as the Polymarket-specific client.
/// All CLOB interactions (auth, order placement, position polling, market cache)
/// live in `crate::service` and are Polymarket-specific implementations.
///
/// Future: extract into a trait impl when a shared `VenueClient` trait is defined.

pub use crate::service::client as clob_client;
pub use crate::service::orders;
pub use crate::service::positions;
pub use crate::service::market_cache;
pub use crate::service::trader;

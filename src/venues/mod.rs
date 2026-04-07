/// Venue adapter layer for multi-platform prediction market support.
///
/// Each sub-module wraps a specific venue's API into a common interface,
/// allowing bots to remain venue-agnostic where possible.
pub mod polymarket;
pub mod kalshi;
pub mod limitless;

/// Common order side representation, shared across venues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Side::Buy => "BUY",
            Side::Sell => "SELL",
        }
    }
}

/// A venue-agnostic market reference.
#[derive(Debug, Clone)]
pub struct MarketRef {
    pub venue: VenueId,
    pub market_id: String,
    pub slug: String,
}

/// Supported prediction market venues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VenueId {
    Polymarket,
    Kalshi,
    Limitless,
}

impl VenueId {
    pub fn as_str(&self) -> &'static str {
        match self {
            VenueId::Polymarket => "Polymarket",
            VenueId::Kalshi => "Kalshi",
            VenueId::Limitless => "Limitless",
        }
    }
}

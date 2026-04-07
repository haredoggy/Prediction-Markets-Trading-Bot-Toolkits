/// Real-time price aggregation across multiple venues.
///
/// Subscribes to price streams from all configured venues and surfaces
/// a unified snapshot. Used by cross-market strategies (e.g. the
/// Cross-Market Arbitrage Bot) that need to compare prices across venues
/// without polling each independently.
///
/// # Status
/// Stub — implementation in progress.

use std::collections::HashMap;
use crate::venues::VenueId;

/// A price snapshot for a single market on a single venue.
#[derive(Debug, Clone)]
pub struct VenuePrice {
    pub venue: VenueId,
    pub market_id: String,
    pub yes_bid: f64,
    pub yes_ask: f64,
    pub no_bid: f64,
    pub no_ask: f64,
    pub timestamp_ms: u64,
}

/// Aggregated price view across all venues for a given market slug.
#[derive(Debug, Clone)]
pub struct AggregatedPrice {
    /// keyed by VenueId
    pub by_venue: HashMap<String, VenuePrice>,
}

impl AggregatedPrice {
    /// Returns the best (cheapest) YES ask across all venues.
    pub fn best_yes_ask(&self) -> Option<(&str, f64)> {
        self.by_venue
            .iter()
            .min_by(|a, b| a.1.yes_ask.partial_cmp(&b.1.yes_ask).unwrap())
            .map(|(venue, p)| (venue.as_str(), p.yes_ask))
    }

    /// Returns the highest YES bid across all venues.
    pub fn best_yes_bid(&self) -> Option<(&str, f64)> {
        self.by_venue
            .iter()
            .max_by(|a, b| a.1.yes_bid.partial_cmp(&b.1.yes_bid).unwrap())
            .map(|(venue, p)| (venue.as_str(), p.yes_bid))
    }

    /// Returns the cross-venue price delta for YES.
    /// Positive = arbitrage opportunity exists (best bid > best ask somewhere).
    pub fn yes_arb_delta(&self) -> f64 {
        let bid = self.best_yes_bid().map(|(_, p)| p).unwrap_or(0.0);
        let ask = self.best_yes_ask().map(|(_, p)| p).unwrap_or(1.0);
        bid - ask
    }
}

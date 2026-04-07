/// Orderbook analysis utilities.
///
/// Shared helpers for computing book metrics used by the
/// Orderbook Imbalance Bot, Market Making Bot, and Spread Farming Bot.

/// A single price level in an orderbook.
#[derive(Debug, Clone)]
pub struct Level {
    pub price: f64,
    pub size_usd: f64,
}

/// A snapshot of one side of the orderbook.
#[derive(Debug, Clone)]
pub struct BookSide {
    pub levels: Vec<Level>,
}

impl BookSide {
    /// Total USD liquidity across all levels.
    pub fn total_depth(&self) -> f64 {
        self.levels.iter().map(|l| l.size_usd).sum()
    }

    /// Depth within `price_range` of the best price.
    pub fn depth_near_top(&self, price_range: f64) -> f64 {
        let Some(best) = self.levels.first() else { return 0.0 };
        self.levels
            .iter()
            .filter(|l| (l.price - best.price).abs() <= price_range)
            .map(|l| l.size_usd)
            .sum()
    }
}

/// Compute the Order Book Imbalance (OBI) as a value in [-1.0, +1.0].
///
/// +1.0 = all liquidity on the bid (extreme buy pressure)
/// -1.0 = all liquidity on the ask (extreme sell pressure)
///  0.0 = perfectly balanced
pub fn obi(bids: &BookSide, asks: &BookSide) -> f64 {
    let bid_depth = bids.total_depth();
    let ask_depth = asks.total_depth();
    let total = bid_depth + ask_depth;
    if total == 0.0 {
        return 0.0;
    }
    (bid_depth - ask_depth) / total
}

/// Returns the mid price given the best bid and ask.
pub fn mid_price(best_bid: f64, best_ask: f64) -> f64 {
    (best_bid + best_ask) / 2.0
}

/// Returns the spread as a fraction of mid price.
pub fn spread_bps(best_bid: f64, best_ask: f64) -> f64 {
    let mid = mid_price(best_bid, best_ask);
    if mid == 0.0 {
        return 0.0;
    }
    ((best_ask - best_bid) / mid) * 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obi_balanced() {
        let bids = BookSide { levels: vec![Level { price: 0.50, size_usd: 1000.0 }] };
        let asks = BookSide { levels: vec![Level { price: 0.52, size_usd: 1000.0 }] };
        assert_eq!(obi(&bids, &asks), 0.0);
    }

    #[test]
    fn obi_bid_heavy() {
        let bids = BookSide { levels: vec![Level { price: 0.50, size_usd: 3000.0 }] };
        let asks = BookSide { levels: vec![Level { price: 0.52, size_usd: 1000.0 }] };
        let v = obi(&bids, &asks);
        assert!(v > 0.0 && v < 1.0);
        // 3000 / 4000 = 0.75 bid share → OBI = (3000-1000)/4000 = 0.5
        assert!((v - 0.5).abs() < 1e-9);
    }
}

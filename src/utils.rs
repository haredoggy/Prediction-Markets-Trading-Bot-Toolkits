//! Misc small helpers.

/// Clamp a Polymarket limit price to the venue's valid band [0.01, 0.99]
/// and round to the 0.001 tick.
pub fn clamp_price(p: f64) -> f64 {
    let p = p.clamp(0.01, 0.99);
    (p * 1000.0).round() / 1000.0
}

/// Convert a USD amount + limit price into an integer number of shares (rounded down).
pub fn usd_to_shares(usd: f64, price: f64) -> f64 {
    if price <= 0.0 {
        return 0.0;
    }
    (usd / price).floor()
}

/// Format a wallet address with the first 6 and last 4 chars, e.g. `0x63ce34…ba9a`.
pub fn truncate_addr(addr: &str) -> String {
    if addr.len() < 10 {
        return addr.to_string();
    }
    let hi = &addr[..6];
    let lo = &addr[addr.len() - 4..];
    format!("{hi}…{lo}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_below_lower() {
        assert!((clamp_price(0.0001) - 0.01).abs() < 1e-9);
    }

    #[test]
    fn clamps_above_upper() {
        assert!((clamp_price(2.0) - 0.99).abs() < 1e-9);
    }

    #[test]
    fn rounds_to_tick() {
        assert!((clamp_price(0.12345) - 0.123).abs() < 1e-9);
    }

    #[test]
    fn usd_to_shares_floor() {
        assert!((usd_to_shares(10.0, 0.33) - 30.0).abs() < 1e-9);
    }

    #[test]
    fn truncate_short_addr_passes_through() {
        assert_eq!(truncate_addr("0x12"), "0x12");
    }
}

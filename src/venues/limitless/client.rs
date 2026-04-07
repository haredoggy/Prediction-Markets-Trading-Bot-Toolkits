/// Limitless on-chain prediction market client.
///
/// Limitless markets live on Base (an Ethereum L2).
/// Order placement interacts with Limitless smart contracts
/// via Alloy; market data is fetched from the Limitless REST API.
///
/// # Status
/// Stub — implementation planned.

use anyhow::Result;

/// Base URL for the Limitless data API.
pub const LIMITLESS_API_BASE: &str = "https://api.limitless.exchange";

/// Minimal Limitless client skeleton.
#[derive(Clone)]
pub struct LimitlessClient {
    pub base_url: String,
    http: reqwest::Client,
}

impl LimitlessClient {
    pub fn new() -> Self {
        Self {
            base_url: LIMITLESS_API_BASE.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Fetch active markets from the Limitless API.
    pub async fn get_markets(&self) -> Result<Vec<LimitlessMarket>> {
        // TODO: implement GET /markets
        anyhow::bail!("Limitless market fetching not yet implemented")
    }
}

impl Default for LimitlessClient {
    fn default() -> Self {
        Self::new()
    }
}

/// A Limitless market.
#[derive(Debug, Clone)]
pub struct LimitlessMarket {
    pub id: String,
    pub title: String,
    pub yes_price: f64,
    pub no_price: f64,
    pub collateral_token: String, // e.g. "USDC"
    pub chain: String,            // e.g. "base"
}

/// Kalshi API client.
///
/// Handles authentication, market data fetching, and order placement
/// against the Kalshi REST API (https://trading-api.kalshi.com).
///
/// # Status
/// Stub — implementation in progress.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Base URL for the Kalshi trading API.
pub const KALSHI_API_BASE: &str = "https://trading-api.kalshi.com/trade-api/v2";

/// Kalshi credentials loaded from config.
#[derive(Debug, Clone)]
pub struct KalshiCredentials {
    pub api_key_id: String,
    pub private_key_pem: String,
}

/// A lightweight Kalshi HTTP client.
///
/// Supports market data queries and order placement via
/// Kalshi's REST API with RSA-based request signing.
#[derive(Clone)]
pub struct KalshiClient {
    pub base_url: String,
    http: reqwest::Client,
    creds: Option<KalshiCredentials>,
}

impl KalshiClient {
    /// Create an unauthenticated client (market data only).
    pub fn new_public() -> Self {
        Self {
            base_url: KALSHI_API_BASE.to_string(),
            http: reqwest::Client::new(),
            creds: None,
        }
    }

    /// Create an authenticated client for order execution.
    pub fn new(creds: KalshiCredentials) -> Self {
        Self {
            base_url: KALSHI_API_BASE.to_string(),
            http: reqwest::Client::new(),
            creds: Some(creds),
        }
    }

    /// Fetch the current YES/NO prices for a market by ticker.
    ///
    /// # Example ticker
    /// `KXBTCD-25FEB2615-T47249.99`
    pub async fn get_market_price(&self, ticker: &str) -> Result<KalshiMarketPrice> {
        let url = format!("{}/markets/{}", self.base_url, ticker);
        let resp = self.http.get(&url).send().await?;
        let body: serde_json::Value = resp.json().await?;
        let market = &body["market"];
        Ok(KalshiMarketPrice {
            ticker: ticker.to_string(),
            yes_bid: market["yes_bid"].as_f64().unwrap_or(0.0) / 100.0,
            yes_ask: market["yes_ask"].as_f64().unwrap_or(0.0) / 100.0,
            no_bid: market["no_bid"].as_f64().unwrap_or(0.0) / 100.0,
            no_ask: market["no_ask"].as_f64().unwrap_or(0.0) / 100.0,
            volume: market["volume"].as_f64().unwrap_or(0.0),
        })
    }

    /// Place an order on Kalshi (requires authenticated client).
    pub async fn place_order(&self, _req: &KalshiOrderRequest) -> Result<KalshiOrderResponse> {
        // TODO: implement RSA-signed POST /portfolio/orders
        anyhow::bail!("Kalshi order placement not yet implemented")
    }
}

/// Real-time price snapshot for a Kalshi market.
#[derive(Debug, Clone, Deserialize)]
pub struct KalshiMarketPrice {
    pub ticker: String,
    /// YES bid (0.0 – 1.0)
    pub yes_bid: f64,
    /// YES ask (0.0 – 1.0)
    pub yes_ask: f64,
    /// NO bid (0.0 – 1.0)
    pub no_bid: f64,
    /// NO ask (0.0 – 1.0)
    pub no_ask: f64,
    pub volume: f64,
}

/// Order request for Kalshi.
#[derive(Debug, Clone, Serialize)]
pub struct KalshiOrderRequest {
    pub ticker: String,
    pub side: String, // "yes" or "no"
    pub action: String, // "buy" or "sell"
    pub count: u64,   // number of contracts
    pub r#type: String, // "market" or "limit"
    pub limit_price: Option<u64>, // in cents (1–99)
}

/// Response from a Kalshi order placement.
#[derive(Debug, Clone, Deserialize)]
pub struct KalshiOrderResponse {
    pub order_id: String,
    pub status: String,
    pub filled_count: u64,
    pub remaining_count: u64,
}

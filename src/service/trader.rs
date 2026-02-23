//! Copy-trade execution service for Polymarket.
//!
//! Executes copy trades on Polymarket based on detected position changes.
//! Thread-safe for parallel execution; caches market_id by slug to reduce latency.

use anyhow::{Context, Result};
use polymarket_client_sdk::auth::Normal;
use polymarket_client_sdk::auth::state::Authenticated;
use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::response::PostOrderResponse;
use polymarket_client_sdk::types::Decimal;
use reqwest::Client as HttpClient;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::config::AppConfig;
use crate::service::client::create_authenticated_clob_client;
use crate::service::orders;
use crate::ui::components::logs::{LogEntry, LogLevel};

/// Copy trader for executing copy trades on Polymarket.
///
/// Thread-safe for parallel execution; caches market_id by slug to reduce latency.
#[derive(Clone)]
pub struct CopyTrader {
    /// Authenticated CLob client
    client: Client<Authenticated<Normal>>,

    /// HTTP client for Gamma API requests
    http_client: HttpClient,

    /// Configuration
    config: AppConfig,

    /// Market ID cache: slug -> market_id
    market_id_cache: Arc<Mutex<HashMap<String, String>>>,

    /// Log sender for logging
    log_tx: Option<tokio::sync::mpsc::UnboundedSender<LogEntry>>,
}

impl CopyTrader {
    /// Create a new CopyTrader instance.
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration
    /// * `log_tx` - Optional log channel sender for logging
    pub async fn new(
        config: &AppConfig,
        log_tx: Option<tokio::sync::mpsc::UnboundedSender<LogEntry>>,
    ) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Polymarket-Toolkits/1.0")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client: create_authenticated_clob_client(
                config.site.clob_api_base.clone(),
                config.bot.private_key.clone(),
                config.bot.funder_address.clone(),
            )
            .await?,
            http_client,
            config: config.clone(),
            market_id_cache: Arc::new(Mutex::new(HashMap::new())),
            log_tx: log_tx.clone(),
        })
    }

    /// Log a message if log channel is available
    fn log(&self, message: String, level: LogLevel) {
        if let Some(ref tx) = self.log_tx {
            let _ = tx.send(LogEntry::new(message, level));
        }
    }

    /// Resolve slug to market_id, using cache when possible.
    ///
    /// # Arguments
    ///
    /// * `slug` - Market slug to resolve
    ///
    /// # Returns
    ///
    /// Market ID if found, None otherwise
    async fn get_market_id(&self, slug: Option<&str>) -> Option<String> {
        let slug = match slug {
            Some(s) => s,
            None => {
                self.log("get_market_id: slug is None".to_string(), LogLevel::Warning);
                return None;
            }
        };

        // Check cache first
        {
            let cache = self.market_id_cache.lock().await;
            if let Some(market_id) = cache.get(slug) {
                self.log(
                    format!("get_market_id: Found cached market_id for slug: {}", slug),
                    LogLevel::Info,
                );
                return Some(market_id.clone());
            }
        }

        // Fetch market from Gamma API using slug
        self.log(
            format!(
                "get_market_id: Fetching market from Gamma API for slug: {}",
                slug
            ),
            LogLevel::Info,
        );

        // Use /markets endpoint with slug query parameter
        let url = format!("{}/markets?slug={}", self.config.site.gamma_api_base, slug);

        let response = match self
            .http_client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                self.log(
                    format!(
                        "get_market_id: HTTP request failed for slug '{}': {}",
                        slug, e
                    ),
                    LogLevel::Error,
                );
                return None;
            }
        };

        let status = response.status();
        if !status.is_success() {
            self.log(
                format!(
                    "get_market_id: Gamma API returned error status {} for slug '{}'",
                    status, slug
                ),
                LogLevel::Error,
            );
            return None;
        }

        // Parse JSON response (should be an array of market objects)
        let markets: Vec<Value> = match response.json().await {
            Ok(m) => m,
            Err(e) => {
                self.log(
                    format!(
                        "get_market_id: Failed to parse JSON response for slug '{}': {}",
                        slug, e
                    ),
                    LogLevel::Error,
                );
                return None;
            }
        };

        // Extract conditionId from first market in the array
        let market_id = match markets.first() {
            Some(market) => {
                match market.get("conditionId") {
                    Some(Value::String(id)) => id.clone(),
                    Some(id) => {
                        // Try to convert to string if it's not already a string
                        id.to_string().trim_matches('"').to_string()
                    }
                    None => {
                        self.log(
                            format!("get_market_id: Market object has no 'conditionId' field for slug: {}", slug),
                            LogLevel::Error,
                        );
                        return None;
                    }
                }
            }
            None => {
                self.log(
                    format!("get_market_id: Empty response array for slug: {}", slug),
                    LogLevel::Warning,
                );
                return None;
            }
        };

        if market_id.is_empty() {
            self.log(
                format!("get_market_id: Empty conditionId for slug: {}", slug),
                LogLevel::Error,
            );
            return None;
        }

        // Cache the result
        {
            let mut cache = self.market_id_cache.lock().await;
            cache.insert(slug.to_string(), market_id.clone());
        }

        self.log(
            format!(
                "get_market_id: Successfully resolved slug '{}' to market_id: {}",
                slug, market_id
            ),
            LogLevel::Info,
        );
        Some(market_id)
    }

    /// Execute a single copy trade from a detected position change.
    ///
    /// # Arguments
    ///
    /// * `change` - HashMap with asset, type (BUY/SELL), size, slug, etc.
    /// * `market_id` - Optional market_id to skip lookup (use when batching to reduce latency)
    ///
    /// # Returns
    ///
    /// Order response on success, None otherwise
    pub async fn execute_trade(
        &self,
        change: &HashMap<String, String>,
        market_id: Option<String>,
    ) -> Option<PostOrderResponse> {
        // Log the incoming change for debugging
        self.log(
            format!(
                "execute_trade called with change keys: {:?}",
                change.keys().collect::<Vec<_>>()
            ),
            LogLevel::Info,
        );

        // Extract change data with proper error handling
        let side = match change.get("type") {
            Some(t) => t.to_lowercase(),
            None => {
                self.log(
                    "execute_trade: Missing 'type' field in change".to_string(),
                    LogLevel::Error,
                );
                return None;
            }
        };

        let asset_id = match change.get("asset") {
            Some(a) => a,
            None => {
                self.log(
                    "execute_trade: Missing 'asset' field in change".to_string(),
                    LogLevel::Error,
                );
                return None;
            }
        };

        let original_size: f64 = match change.get("size") {
            Some(s) => match s.parse() {
                Ok(size) => size,
                Err(e) => {
                    self.log(
                        format!("execute_trade: Failed to parse size '{}': {}", s, e),
                        LogLevel::Error,
                    );
                    return None;
                }
            },
            None => {
                self.log(
                    "execute_trade: Missing 'size' field in change".to_string(),
                    LogLevel::Error,
                );
                return None;
            }
        };

        let slug = change.get("slug");

        // Calculate our trade size based on copy percentage
        let our_size =
            (original_size * self.config.trading.copy_percentage / 100.0 * 100.0).round() / 100.0;

        self.log(
            format!("execute_trade: Calculated our_size: {} (from original_size: {}, copy_percentage: {}%)", 
                our_size, original_size, self.config.trading.copy_percentage),
            LogLevel::Info,
        );

        if our_size <= 0.0 {
            self.log(
                format!("Skipping trade: calculated size {} is too small (original_size: {}, copy_percentage: {}%)", 
                    our_size, original_size, self.config.trading.copy_percentage),
                LogLevel::Warning,
            );
            return None;
        }

        // Get market_id if not provided
        let market_id = if let Some(id) = market_id {
            self.log(
                format!("execute_trade: Using provided market_id: {}", id),
                LogLevel::Info,
            );
            id
        } else {
            let slug_str = slug.map(|s| s.as_str());
            self.log(
                format!(
                    "execute_trade: Looking up market_id for slug: {:?}",
                    slug_str
                ),
                LogLevel::Info,
            );
            match self.get_market_id(slug_str).await {
                Some(id) => {
                    self.log(
                        format!("execute_trade: Found market_id: {}", id),
                        LogLevel::Info,
                    );
                    id
                }
                None => {
                    self.log(
                        format!(
                            "execute_trade: Failed to get market_id for slug: {:?}",
                            slug_str
                        ),
                        LogLevel::Error,
                    );
                    return None;
                }
            }
        };

        if market_id.is_empty() {
            self.log(
                format!(
                    "Skipping trade: market_id is empty for slug: {}",
                    slug.map(|s| s.as_str()).unwrap_or("unknown")
                ),
                LogLevel::Warning,
            );
            return None;
        }

        let slug_display = slug.map(|s| s.as_str()).unwrap_or("unknown");
        self.log(
            format!("Copying {} for {}: {} shares", side, slug_display, our_size),
            LogLevel::Info,
        );

        if !self.config.bot.enable_trading {
            self.log(
                "Trading disabled in config (dry run)".to_string(),
                LogLevel::Warning,
            );
            return None;
        }

        // Execute the trade based on side
        self.log(
            format!(
                "execute_trade: Executing {} order for asset_id: {}, size: {}",
                side, asset_id, our_size
            ),
            LogLevel::Info,
        );

        let result = if side == "buy" {
            // Buy order - use USDC amount
            let usdc_amount = Decimal::try_from(our_size).unwrap_or_else(|_| Decimal::from(1u64)); // Fallback to $1.00 if conversion fails
            orders::buy_order(
                &self.config.bot.private_key,
                &self.config.bot.funder_address,
                asset_id,
                usdc_amount,
                Some(polymarket_client_sdk::clob::types::OrderType::FOK),
            )
            .await
        } else if side == "sell" {
            // Sell order - use share amount
            // For market orders, use a low price (0.01) to ensure quick fill
            let size_decimal = Decimal::try_from(our_size).unwrap_or_else(|_| Decimal::from(1u64)); // Fallback to $1.00 if conversion fails
            // Create Decimal for 0.01 (market price for quick fill)
            let market_price = Decimal::from(1) / Decimal::from(100);
            orders::sell_order(
                &self.config.bot.private_key,
                &self.config.bot.funder_address,
                asset_id,
                size_decimal,
                market_price,
                Some(polymarket_client_sdk::clob::types::OrderType::FOK),
            )
            .await
        } else {
            self.log(format!("Unknown trade side: {}", side), LogLevel::Warning);
            return None;
        };

        match result {
            Ok(order_response) => {
                let order_id = order_response.order_id.to_string();
                self.log(
                    format!("Order placed successfully. Order ID: {}", order_id),
                    LogLevel::Success,
                );
                Some(order_response)
            }
            Err(e) => {
                self.log(
                    format!("Failed to execute copy trade: {}", e),
                    LogLevel::Error,
                );
                None
            }
        }
    }
}

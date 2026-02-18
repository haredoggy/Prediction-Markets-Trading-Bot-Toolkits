//! Unified configuration management for the Polymarket trading bot.
//!
//! This module provides a single, comprehensive configuration structure that consolidates
//! all bot settings, trading parameters, site endpoints, and risk management configuration.
pub mod settings;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::utils::risk_guard;

// ============================================================================
// Unified Configuration Structure
// ============================================================================

/// Main application configuration structure.
///
/// Consolidates all configuration needed to run the trading bot, organized into
/// logical sections: bot settings, site endpoints, trading parameters, and risk management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Bot-specific configuration (wallet, credentials, API keys)
    #[serde(default)]
    pub bot: BotConfig,
    
    /// Site endpoints and API URLs
    #[serde(default)]
    pub site: SiteConfig,
    
    /// Trading parameters and execution settings
    #[serde(default)]
    pub trading: TradingConfig,
    
    /// Risk management and circuit breaker settings
    #[serde(default)]
    pub risk: RiskConfig,
}

// ============================================================================
// Bot Configuration
// ============================================================================

/// Bot-specific configuration including wallet credentials and API keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// Target wallet address for monitoring
    #[serde(default = "default_zero_address")]
    pub target_wallet: String,
    
    /// Alchemy API key for blockchain data access
    #[serde(default)]
    pub alchemy_api_key: String,
    
    /// Private key for signing transactions (64-character hex, no 0x prefix)
    #[serde(default)]
    pub private_key: String,
    
    /// Proxy wallet address (funder) for the account
    #[serde(default = "default_zero_address")]
    pub funder_address: String,
    
    /// Whether trading is enabled
    #[serde(default = "default_true")]
    pub enable_trading: bool,
    
    /// Whether to use mock trading mode (no real orders)
    #[serde(default = "default_false")]
    pub mock_trading: bool,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            target_wallet: default_zero_address(),
            alchemy_api_key: String::new(),
            private_key: String::new(),
            funder_address: default_zero_address(),
            enable_trading: true,
            mock_trading: false,
        }
    }
}

// ============================================================================
// Site Configuration
// ============================================================================

/// Site endpoints and API URLs configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    /// Gamma API base URL for market data
    #[serde(default = "default_gamma_api_base")]
    pub gamma_api_base: String,
    
    /// CLOB API base URL for order operations
    #[serde(default = "default_clob_api_base")]
    pub clob_api_base: String,
    
    /// CLOB WebSocket URL for real-time order updates
    #[serde(default = "default_clob_wss_url")]
    pub clob_wss_url: String,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            gamma_api_base: default_gamma_api_base(),
            clob_api_base: default_clob_api_base(),
            clob_wss_url: default_clob_wss_url(),
        }
    }
}

impl SiteConfig {
    /// Returns the Gamma API base URL
    pub fn gamma_api_base(&self) -> &str {
        &self.gamma_api_base
    }
    
    /// Returns the CLOB API base URL
    pub fn clob_api_base(&self) -> &str {
        &self.clob_api_base
    }
    
    /// Returns the CLOB WebSocket URL
    pub fn clob_wss_url(&self) -> &str {
        &self.clob_wss_url
    }
}

// ============================================================================
// Trading Configuration
// ============================================================================

/// Trading parameters and execution settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    /// Price buffer for order placement
    #[serde(default = "default_price_buffer")]
    pub price_buffer: f64,
    
    /// Scaling ratio for order size calculation
    #[serde(default = "default_scaling_ratio")]
    pub scaling_ratio: f64,
    
    /// Minimum cash value per trade (USD)
    #[serde(default = "default_min_cash_value")]
    pub min_cash_value: f64,
    
    /// Minimum share count per trade
    #[serde(default = "default_min_share_count")]
    pub min_share_count: f64,
    
    /// Whether to use probabilistic sizing
    #[serde(default = "default_true")]
    pub use_probabilistic_sizing: bool,
    
    /// Fixed trade value in USD (0.0 = disabled, >0 = fixed dollar amount per trade)
    #[serde(default = "default_fixed_trade_value")]
    pub fixed_trade_value: f64,
    
    /// Minimum whale trade size to copy (skip trades below this)
    #[serde(default = "default_min_whale_shares")]
    pub min_whale_shares_to_copy: f64,
    
    /// Price increment for resubmit attempts
    #[serde(default = "default_resubmit_price_increment")]
    pub resubmit_price_increment: f64,
    
    /// Order reply timeout
    #[serde(default = "default_order_reply_timeout_secs")]
    pub order_reply_timeout_secs: u64,
    
    /// Book request timeout (milliseconds)
    #[serde(default = "default_book_req_timeout_ms")]
    pub book_req_timeout_ms: u64,
    
    /// WebSocket ping timeout (seconds)
    #[serde(default = "default_ws_ping_timeout_secs")]
    pub ws_ping_timeout_secs: u64,
    
    /// WebSocket reconnect delay (seconds)
    #[serde(default = "default_ws_reconnect_delay_secs")]
    pub ws_reconnect_delay_secs: u64,
    
    /// GTD expiry seconds for live markets
    #[serde(default = "default_gtd_expiry_live_secs")]
    pub gtd_expiry_live_secs: u64,
    
    /// GTD expiry seconds for closed markets
    #[serde(default = "default_gtd_expiry_closed_secs")]
    pub gtd_expiry_closed_secs: u64,
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            price_buffer: default_price_buffer(),
            scaling_ratio: default_scaling_ratio(),
            min_cash_value: default_min_cash_value(),
            min_share_count: default_min_share_count(),
            use_probabilistic_sizing: true,
            fixed_trade_value: default_fixed_trade_value(),
            min_whale_shares_to_copy: default_min_whale_shares(),
            resubmit_price_increment: default_resubmit_price_increment(),
            order_reply_timeout_secs: default_order_reply_timeout_secs(),
            book_req_timeout_ms: default_book_req_timeout_ms(),
            ws_ping_timeout_secs: default_ws_ping_timeout_secs(),
            ws_reconnect_delay_secs: default_ws_reconnect_delay_secs(),
            gtd_expiry_live_secs: default_gtd_expiry_live_secs(),
            gtd_expiry_closed_secs: default_gtd_expiry_closed_secs(),
        }
    }
}

impl TradingConfig {
    /// Returns true if this trade should be skipped (too small)
    #[inline]
    pub fn should_skip_trade(&self, whale_shares: f64) -> bool {
        whale_shares < self.min_whale_shares_to_copy
    }
    
    /// Returns GTD expiry seconds based on market liveness
    #[inline]
    pub fn get_gtd_expiry_secs(&self, is_live: bool) -> u64 {
        if is_live {
            self.gtd_expiry_live_secs
        } else {
            self.gtd_expiry_closed_secs
        }
    }
    
    /// Returns order reply timeout as Duration
    #[inline]
    pub fn order_reply_timeout(&self) -> Duration {
        Duration::from_secs(self.order_reply_timeout_secs)
    }
    
    /// Returns book request timeout as Duration
    #[inline]
    pub fn book_req_timeout(&self) -> Duration {
        Duration::from_millis(self.book_req_timeout_ms)
    }
    
    /// Returns WebSocket ping timeout as Duration
    #[inline]
    pub fn ws_ping_timeout(&self) -> Duration {
        Duration::from_secs(self.ws_ping_timeout_secs)
    }
    
    /// Returns WebSocket reconnect delay as Duration
    #[inline]
    pub fn ws_reconnect_delay(&self) -> Duration {
        Duration::from_secs(self.ws_reconnect_delay_secs)
    }
}

// ============================================================================
// Risk Configuration
// ============================================================================

/// Risk management and circuit breaker configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Minimum share count to be considered a "large trade" for circuit breaker
    #[serde(default = "default_large_trade_shares")]
    pub large_trade_shares: f64,
    
    /// Number of consecutive large trades needed to trigger circuit breaker
    #[serde(default = "default_consecutive_trigger")]
    pub consecutive_trigger: u8,
    
    /// Time window in seconds for tracking consecutive trades
    #[serde(default = "default_sequence_window_secs")]
    pub sequence_window_secs: u64,
    
    /// Minimum orderbook depth in USD required beyond our order size
    #[serde(default = "default_min_depth_usd")]
    pub min_depth_usd: f64,
    
    /// Duration in seconds that circuit breaker stays tripped after activation
    #[serde(default = "default_trip_duration_secs")]
    pub trip_duration_secs: u64,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            large_trade_shares: default_large_trade_shares(),
            consecutive_trigger: default_consecutive_trigger(),
            sequence_window_secs: default_sequence_window_secs(),
            min_depth_usd: default_min_depth_usd(),
            trip_duration_secs: default_trip_duration_secs(),
        }
    }
}

impl RiskConfig {
    /// Converts risk configuration to risk guard configuration.
    pub fn to_risk_guard_config(&self) -> risk_guard::RiskGuardConfig {
        risk_guard::RiskGuardConfig {
            large_trade_shares: self.large_trade_shares,
            consecutive_trigger: self.consecutive_trigger,
            sequence_window: Duration::from_secs(self.sequence_window_secs),
            min_depth_beyond_usd: self.min_depth_usd,
            trip_duration: Duration::from_secs(self.trip_duration_secs),
        }
    }
}

// ============================================================================
// AppConfig Implementation
// ============================================================================

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            bot: BotConfig::default(),
            site: SiteConfig::default(),
            trading: TradingConfig::default(),
            risk: RiskConfig::default(),
        }
    }
}

impl AppConfig {
    /// Loads configuration from a YAML file.
    ///
    /// # Environment Variables
    ///
    /// - `CONFIG`: Path to config file (defaults to "config.yaml")
    ///
    /// # Returns
    ///
    /// Returns the loaded configuration or default configuration if file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let config_path = std::env::var("CONFIG").unwrap_or_else(|_| "config.yaml".to_string());

        if Path::new(&config_path).exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: AppConfig = serde_yaml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
    
    /// Converts risk configuration to risk guard configuration.
    pub fn risk_guard_config(&self) -> risk_guard::RiskGuardConfig {
        self.risk.to_risk_guard_config()
    }
}

// ============================================================================
// Default Value Helper Functions
// ============================================================================

fn default_zero_address() -> String {
    "0x0000000000000000000000000000000000000000".to_string()
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

// Site defaults
fn default_gamma_api_base() -> String {
    "https://gamma-api.polymarket.com".to_string()
}

fn default_clob_api_base() -> String {
    "https://clob.polymarket.com".to_string()
}

fn default_clob_wss_url() -> String {
    "wss://clob.polymarket.com".to_string()
}

// Trading defaults
fn default_price_buffer() -> f64 {
    0.00
}

fn default_scaling_ratio() -> f64 {
    1.00
}

fn default_min_cash_value() -> f64 {
    0.00
}

fn default_min_share_count() -> f64 {
    0.0
}

fn default_fixed_trade_value() -> f64 {
    1.00
}

fn default_min_whale_shares() -> f64 {
    0.0
}

fn default_resubmit_price_increment() -> f64 {
    0.01
}

fn default_order_reply_timeout_secs() -> u64 {
    10
}

fn default_book_req_timeout_ms() -> u64 {
    2500
}

fn default_ws_ping_timeout_secs() -> u64 {
    300
}

fn default_ws_reconnect_delay_secs() -> u64 {
    3
}

fn default_gtd_expiry_live_secs() -> u64 {
    61
}

fn default_gtd_expiry_closed_secs() -> u64 {
    1800
}

// Risk defaults
fn default_large_trade_shares() -> f64 {
    1500.0
}

fn default_consecutive_trigger() -> u8 {
    2
}

fn default_sequence_window_secs() -> u64 {
    30
}

fn default_min_depth_usd() -> f64 {
    200.0
}

fn default_trip_duration_secs() -> u64 {
    120
}
/// Settings and configuration management
/// Handles environment variable loading and validation

use anyhow::{Context, Result};
use std::env;
use std::path::Path;
use std::time::Duration;
use crate::utils::risk_guard;

// ============================================================================
// Blockchain Constants
// ============================================================================

use once_cell::sync::Lazy;

pub const ORDERS_FILLED_EVENT_SIGNATURE: &str =
    "0xd0a08e8c493f9c94f29311604c9de1b4e8c8d4c06bd0c789af57f2d65bfec0f6";

/// Target whale address topic - loaded from TARGET_WHALE_ADDRESS env var
/// Format: 40-char hex address without 0x prefix (e.g., "204f72f35326db932158cba6adff0b9a1da95e14")
/// Gets zero-padded to 66 chars with 0x prefix for topic matching
/// 
/// Note: This is validated in Config::from_env() before use, so expect should not panic
/// in normal operation. If you see this panic, it means Config::from_env() was not called first.
pub static TARGET_TOPIC_HEX: Lazy<String> = Lazy::new(|| {
    let addr = env::var("TARGET_WHALE_ADDRESS")
        .expect("TARGET_WHALE_ADDRESS should have been validated in Config::from_env(). \
                If you see this, please ensure you call Config::from_env() before using TARGET_TOPIC_HEX");
    format!("0x000000000000000000000000{}", addr.trim_start_matches("0x").to_lowercase())
});

pub const MONITORED_ADDRESSES: [&str; 3] = [
    "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E",
    "0x4d97dcd97ec945f40cf65f87097ace5ea0476045",
    "0xC5d563A36AE78145C45a50134d48A1215220f80a",
];

// ============================================================================
// API & File Constants
// ============================================================================

// Debug flag - set to true to print full API error messages (remove after debugging)
pub const DEBUG_FULL_ERRORS: bool = true;

// ============================================================================
// Trading Constants
// ============================================================================

pub const PRICE_BUFFER: f64 = 0.00;
pub const SCALING_RATIO: f64 = 1.00;
pub const MIN_CASH_VALUE: f64 = 0.00;
pub const MIN_SHARE_COUNT: f64 = 0.0;  // Set to 0 to rely purely on MIN_CASH_VALUE for EV scaling
pub const USE_PROBABILISTIC_SIZING: bool = true;

// Fixed trade value: set to 1.00 for $1 per trade (tx tracker wallet style)
// When set > 0, ignores SCALING_RATIO and uses fixed dollar amount per trade
pub const FIXED_TRADE_VALUE: f64 = 1.00;

// Minimum whale trade size to copy (skip trades below this)
pub const MIN_WHALE_SHARES_TO_COPY: f64 = 0.0;

/// Returns true if this trade should be skipped (too small, negative expected value)
#[inline]
pub fn should_skip_trade(whale_shares: f64) -> bool {
    whale_shares < MIN_WHALE_SHARES_TO_COPY
}

// ============================================================================
// Timeouts
// ============================================================================

pub const ORDER_REPLY_TIMEOUT: Duration = Duration::from_secs(10);

// ============================================================================
// Resubmitter Configuration (for FAK failures)
// ============================================================================

pub const RESUBMIT_PRICE_INCREMENT: f64 = 0.01;

// Tier-based max resubmit attempts (4000+ gets 5, others get 4)
#[inline]
pub fn get_max_resubmit_attempts(whale_shares: f64) -> u8 {
    if whale_shares >= 4000.0 { 5 }
    else { 4 }
}

/// Returns true if this attempt should increment price, false for flat retry
/// >= 4000: chase attempt 1 only
/// <4000: never chase (buffer=0)
#[inline]
pub fn should_increment_price(whale_shares: f64, attempt: u8) -> bool {
    if whale_shares >= 4000.0 {
        attempt == 1  
    } else {
        false  
    }
}

#[inline]
pub fn get_gtd_expiry_secs(is_live: bool) -> u64 {
    if is_live { 61 }    
    else { 1800 }        
}

// Tier-based max buffer for resubmits (on top of initial tier buffer)
// >= 4000: chase up to +0.02
// <4000: no chasing (0.00)
#[inline]
pub fn get_resubmit_max_buffer(whale_shares: f64) -> f64 {
    if whale_shares >= 4000.0 { 0.01 }
    else { 0.00 }
}
pub const BOOK_REQ_TIMEOUT: Duration = Duration::from_millis(2500);
pub const WS_PING_TIMEOUT: Duration = Duration::from_secs(300);
pub const WS_RECONNECT_DELAY: Duration = Duration::from_secs(3);

// ============================================================================
// Execution Tiers
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct ExecutionTier {
    pub min_shares: f64,
    pub price_buffer: f64,
    pub order_action: &'static str,
    pub size_multiplier: f64,
}

pub const EXECUTION_TIERS: [ExecutionTier; 3] = [
    ExecutionTier {
        min_shares: 4000.0,
        price_buffer: 0.01,
        order_action: "FAK",
        size_multiplier: 1.25,
    },
    ExecutionTier {
        min_shares: 2000.0,
        price_buffer: 0.01,
        order_action: "FAK",
        size_multiplier: 1.0,
    },
    ExecutionTier {
        min_shares: 1000.0,
        price_buffer: 0.00,
        order_action: "FAK",
        size_multiplier: 1.0,
    },
];

/// Get tier params for a given trade size
/// Returns (buffer, order_action, size_multiplier)
#[inline]
pub fn get_tier_params(whale_shares: f64, side_is_buy: bool, token_id: &str) -> (f64, &'static str, f64) {
    if !side_is_buy {
        return (PRICE_BUFFER, "GTD", 1.0);
    }

    // Get base tier params - direct if-else is faster than iterator for 3 tiers
    let (base_buffer, order_action, size_multiplier) = if whale_shares >= 4000.0 {
        (0.01, "FAK", 1.25)
    } else if whale_shares >= 2000.0 {
        (0.01, "FAK", 1.0)
    } else if whale_shares >= 1000.0 {
        (0.0, "FAK", 1.0)
    } else {
        (PRICE_BUFFER, "FAK", 1.0)  // Small buys use FAK (Fill and Kill)
    };

    (base_buffer, order_action, size_multiplier)
}

// ============================================================================
// Runtime Configuration (loaded from environment)
// ============================================================================

#[derive(Debug, Clone)]
pub struct Config {
    // Credentials
    pub private_key: String,
    pub funder_address: String,
    
    // WebSocket
    pub wss_url: String,
    
    // Trading flags
    pub enable_trading: bool,
    pub mock_trading: bool,
    
    // Circuit breaker
    pub cb_large_trade_shares: f64,
    pub cb_consecutive_trigger: u8,
    pub cb_sequence_window_secs: u64,
    pub cb_min_depth_usd: f64,
    pub cb_trip_duration_secs: u64,
}

impl Config {
    /// Load configuration from environment variables
    /// 
    /// # Errors
    /// 
    /// Returns errors with helpful messages if required configuration is missing or invalid.
    /// For detailed setup help, see docs/02_SETUP_GUIDE.md
    pub fn from_env() -> Result<Self> {
        // Check if .env file exists (helpful error for beginners)
        if !Path::new(".env").exists() {
            anyhow::bail!(
                "Configuration file .env not found!\n\
                \n\
                Setup steps:\n\
                1. Copy .env.example to .env\n\
                2. Open .env in a text editor\n\
                3. Fill in your configuration values\n\
                    4. See docs/02_SETUP_GUIDE.md for detailed instructions\n\
                \n\
                Quick check: Run 'cargo run --release --bin check_config' to validate your setup"
            );
        }
        
        let private_key = env::var("PRIVATE_KEY")
            .context("PRIVATE_KEY env var is required. Add it to your .env file.\n\
                     Format: 64-character hex string (no 0x prefix)\n\
                     Example: 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")?;
        
        // Validate private key format
        let key_clean = private_key.trim().strip_prefix("0x").unwrap_or(private_key.trim());
        if key_clean.len() != 64 {
            anyhow::bail!(
                "PRIVATE_KEY must be exactly 64 hex characters (found {}).\n\
                Remove any '0x' prefix. Current value starts with: {}",
                key_clean.len(),
                if key_clean.len() > 10 { format!("{}...", &key_clean[..10]) } else { key_clean.to_string() }
            );
        }
        if !key_clean.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("PRIVATE_KEY contains invalid characters. Must be hexadecimal (0-9, a-f, A-F).");
        }
        
        let funder_address = env::var("FUNDER_ADDRESS")
            .context("FUNDER_ADDRESS env var is required. Add it to your .env file.\n\
                     Format: 40-character hex address (can include 0x prefix)\n\
                     This should match the wallet from your PRIVATE_KEY")?;
        
        // Validate funder address format
        let addr_clean = funder_address.trim().strip_prefix("0x").unwrap_or(funder_address.trim());
        if addr_clean.len() != 40 {
            anyhow::bail!(
                "FUNDER_ADDRESS must be exactly 40 hex characters (found {}).\n\
                Current value: {}",
                addr_clean.len(),
                if addr_clean.len() > 20 { format!("{}...", &addr_clean[..20]) } else { addr_clean.to_string() }
            );
        }
        if !addr_clean.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("FUNDER_ADDRESS contains invalid characters. Must be hexadecimal (0-9, a-f, A-F).");
        }
        
        // WebSocket URL from either provider
        let wss_url = if let Ok(key) = env::var("ALCHEMY_API_KEY") {
            let key = key.trim();
            if key.is_empty() || key == "your_alchemy_api_key_here" {
                anyhow::bail!(
                    "ALCHEMY_API_KEY is set but has placeholder value.\n\
                    Get your API key from https://www.alchemy.com/ (free tier available)\n\
                    Then add it to your .env file"
                );
            }
            format!("wss://polygon-mainnet.g.alchemy.com/v2/{}", key)
        } else if let Ok(key) = env::var("CHAINSTACK_API_KEY") {
            let key = key.trim();
            if key.is_empty() || key == "your_chainstack_api_key_here" {
                anyhow::bail!(
                    "CHAINSTACK_API_KEY is set but has placeholder value.\n\
                    Get your API key from https://chainstack.com/ (free tier available)\n\
                    Or use ALCHEMY_API_KEY instead (recommended for beginners)"
                );
            }
            format!("wss://polygon-mainnet.core.chainstack.com/{}", key)
        } else {
            anyhow::bail!(
                "WebSocket API key required!\n\
                \n\
                Set either ALCHEMY_API_KEY or CHAINSTACK_API_KEY in your .env file.\n\
                \n\
                Recommended (beginners): ALCHEMY_API_KEY\n\
                1. Sign up at https://www.alchemy.com/\n\
                2. Create app (Polygon Mainnet)\n\
                3. Copy API key to .env file\n\
                \n\
                Alternative: CHAINSTACK_API_KEY\n\
                1. Sign up at https://chainstack.com/\n\
                2. Create Polygon node\n\
                3. Copy API key to .env file\n\
                \n\
                Run 'cargo run --release --bin check_config' to validate your setup"
            );
        };
        
        // Validate TARGET_WHALE_ADDRESS (used by TARGET_TOPIC_HEX lazy static)
        let target_whale = env::var("TARGET_WHALE_ADDRESS")
            .context("TARGET_WHALE_ADDRESS env var is required. Add it to your .env file.\n\
                     Format: 40-character hex address (no 0x prefix)\n\
                     This is the whale address you want to copy trades from.\n\
                     Find whale addresses on Polymarket leaderboards")?;
        
        let whale_clean = target_whale.trim().strip_prefix("0x").unwrap_or(target_whale.trim());
        if whale_clean.is_empty() || whale_clean == "target_whale_address_here" {
            anyhow::bail!(
                "TARGET_WHALE_ADDRESS is set but has placeholder value.\n\
                Replace 'target_whale_address_here' with the actual whale address you want to copy.\n\
                Find whale addresses on Polymarket leaderboards or from successful traders."
            );
        }
        if whale_clean.len() != 40 {
            anyhow::bail!(
                "TARGET_WHALE_ADDRESS must be exactly 40 hex characters (found {}).\n\
                Remove '0x' prefix if present. Current value: {}",
                whale_clean.len(),
                if whale_clean.len() > 20 { format!("{}...", &whale_clean[..20]) } else { whale_clean.to_string() }
            );
        }
        if !whale_clean.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("TARGET_WHALE_ADDRESS contains invalid characters. Must be hexadecimal (0-9, a-f, A-F).");
        }
        
        let enable_trading = env::var("ENABLE_TRADING")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(true);
        
        let mock_trading = env::var("MOCK_TRADING")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);
        
        Ok(Self {
            private_key,
            funder_address,
            wss_url,
            enable_trading,
            mock_trading,
            cb_large_trade_shares: env_parse("CB_LARGE_TRADE_SHARES", 1500.0),
            cb_consecutive_trigger: env_parse("CB_CONSECUTIVE_TRIGGER", 2u8),
            cb_sequence_window_secs: env_parse("CB_SEQUENCE_WINDOW_SECS", 30),
            cb_min_depth_usd: env_parse("CB_MIN_DEPTH_USD", 200.0),
            cb_trip_duration_secs: env_parse("CB_TRIP_DURATION_SECS", 120),
        })
    }
    
    /// Convert to RiskGuardConfig for safety checks
    pub fn risk_guard_config(&self) -> risk_guard::RiskGuardConfig {
        risk_guard::RiskGuardConfig {
            large_trade_shares: self.cb_large_trade_shares,
            consecutive_trigger: self.cb_consecutive_trigger,
            sequence_window: Duration::from_secs(self.cb_sequence_window_secs),
            min_depth_beyond_usd: self.cb_min_depth_usd,
            trip_duration: Duration::from_secs(self.cb_trip_duration_secs),
        }
    }
}

/// Parse env var with default fallback
fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
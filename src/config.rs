//! Configuration loading and types.
//!
//! Two-file split:
//! - `config.json` — public settings (committed)
//! - `config.yaml` — credentials (gitignored, must never be committed)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub bot: BotConfig,
    pub site: SiteConfig,
    pub strategy: StrategyConfig,
    pub trading: TradingConfig,
    pub risk: RiskConfig,
    pub exchange: ExchangeConfig,

    /// Loaded from `config.yaml`. Not serialised back out.
    #[serde(skip)]
    pub credentials: Credentials,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub wallets_to_track: Vec<String>,

    /// When `false`, decisions are computed but no orders are sent.
    #[serde(default)]
    pub enable_trading: bool,

    /// When `true`, every order path returns early with a log line.
    /// Independent of `enable_trading` — both must be permissive for live trades.
    #[serde(default = "default_true")]
    pub mock_trading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    pub gamma_api_base: String,
    pub data_api_base: String,
    pub clob_api_base: String,
    pub clob_wss_url: String,
    pub polygon_rpc_url: String,
    pub polygon_ws_url: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum CopyStrategy {
    Percentage,
    Fixed,
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub copy_strategy: CopyStrategy,

    /// For PERCENTAGE: percent of whale notional (e.g. 20.0 = 20%).
    /// For FIXED: USD per copy leg.
    /// For ADAPTIVE: ignored (uses adaptive_* below).
    pub copy_size: f64,

    pub trade_multiplier: f64,
    pub min_order_size_usd: f64,
    pub max_order_size_usd: f64,
    pub min_whale_shares_to_copy: f64,
    pub adaptive_threshold_usd: f64,
    pub adaptive_min_percent: f64,
    pub adaptive_max_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    /// Max API requests per `rate_window_secs`.
    pub rate_limit: u32,
    pub rate_window_secs: u64,
    pub poll_interval_secs: u64,

    /// Slippage tolerance applied to the order limit price (fractional, e.g. 0.005 = 0.5¢ at $1 scale).
    pub price_buffer: f64,

    pub fee_rate_bps: u32,
    pub order_expiration_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub large_trade_shares: f64,
    pub consecutive_trigger: u32,
    pub sequence_window_secs: u64,
    pub min_depth_usd: f64,
    pub trip_duration_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub ctf_exchange_address: String,
    pub neg_risk_exchange_address: String,
    pub chain_id: u64,
    pub domain_name: String,
    pub domain_version: String,
}

#[derive(Debug, Clone, Default)]
pub struct Credentials {
    pub private_key: String,
    pub funder_address: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub api_passphrase: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CredentialsFile {
    bot: CredentialsBot,
}

#[derive(Debug, Deserialize)]
struct CredentialsBot {
    private_key: String,
    funder_address: String,
    api_key: Option<String>,
    api_secret: Option<String>,
    api_passphrase: Option<String>,
}

impl AppConfig {
    pub fn load(config_path: &Path, credentials_path: &Path) -> Result<Self> {
        let raw =
            std::fs::read_to_string(config_path).with_context(|| {
                format!("reading public config from {}", config_path.display())
            })?;
        let mut cfg: AppConfig =
            serde_json::from_str(&raw).context("parsing config.json")?;

        // Credentials file is optional — without it we can still run in mock mode.
        if credentials_path.exists() {
            let raw = std::fs::read_to_string(credentials_path).with_context(|| {
                format!("reading credentials from {}", credentials_path.display())
            })?;
            let parsed: CredentialsFile =
                serde_yaml::from_str(&raw).context("parsing config.yaml")?;
            cfg.credentials = Credentials {
                private_key: parsed.bot.private_key,
                funder_address: parsed.bot.funder_address,
                api_key: parsed.bot.api_key,
                api_secret: parsed.bot.api_secret,
                api_passphrase: parsed.bot.api_passphrase,
            };
        }

        // Environment-variable overrides (handy for CI / Docker).
        if let Ok(key) = std::env::var("PM_PRIVATE_KEY") {
            cfg.credentials.private_key = key;
        }
        if let Ok(addr) = std::env::var("PM_FUNDER_ADDRESS") {
            cfg.credentials.funder_address = addr;
        }

        Ok(cfg)
    }

    pub fn live_trading_allowed(&self) -> bool {
        self.bot.enable_trading && !self.bot.mock_trading
    }
}

fn default_true() -> bool {
    true
}

/// PM Whale Follower - Main entry point
/// Monitors blockchain for whale trades and executes copy trades

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use alloy::primitives::U256;
use futures::{SinkExt, StreamExt};
use rand::Rng;
use pm_whale_follower::{ApiCreds, OrderArgs, RustClobClient, PreparedCreds, OrderResponse};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

mod models;

use pm_whale_follower::risk_guard::{RiskGuard, RiskGuardConfig, SafetyDecision, TradeSide, calc_liquidity_depth};
use pm_whale_follower::settings::*;
use pm_whale_follower::market_cache;
use pm_whale_follower::tennis_markets;
use pm_whale_follower::soccer_markets;
use pm_whale_follower::orders;
use polymarket_client_sdk::clob::types::OrderType;
use polymarket_client_sdk::types::Decimal;
use std::sync::Arc;
use models::*;

const GAMMA_API_BASE: &str = "https://gamma-api.polymarket.com";

// ============================================================================
// Thread-local buffers 
// ============================================================================

thread_local! {
    static CSV_BUF: RefCell<String> = RefCell::new(String::with_capacity(512));
    static SANITIZE_BUF: RefCell<String> = RefCell::new(String::with_capacity(128));
    static TOKEN_ID_CACHE: RefCell<HashMap<[u8; 32], Arc<str>>> = RefCell::new(HashMap::with_capacity(256));
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    ensure_csv()?;

    // Initialize market data caches
    market_cache::init_caches();

    // Start background cache refresh task
    let _cache_refresh_handle = market_cache::spawn_cache_refresh_task();

    let cfg = Config::from_env()?;
    
    let (client, creds) = build_worker_state(
        cfg.private_key.clone(),
        cfg.funder_address.clone(),
        ".clob_market_cache.json",
        ".clob_creds.json",
    ).await?;
    
    let prepared_creds = PreparedCreds::from_api_creds(&creds)?;
    let risk_config = cfg.risk_guard_config();

    let (order_tx, order_rx) = mpsc::channel(1024);
    let (resubmit_tx, resubmit_rx) = mpsc::unbounded_channel::<ResubmitRequest>();

    let client_arc = Arc::new(client);
    let creds_arc = Arc::new(prepared_creds.clone());
    let private_key_arc = Arc::new(cfg.private_key.clone());
    let funder_address_arc = Arc::new(cfg.funder_address.clone());

    start_order_worker(order_rx, client_arc.clone(), private_key_arc.clone(), funder_address_arc.clone(), cfg.enable_trading, cfg.mock_trading, risk_config, resubmit_tx.clone());

    tokio::spawn(resubmit_worker(resubmit_rx, client_arc, creds_arc));

    let order_engine = OrderEngine {
        tx: order_tx,
        resubmit_tx,
        enable_trading: cfg.enable_trading,
    };

    println!(
        "🚀 Starting trader. Trading: {}, Mock: {}",
        cfg.enable_trading, cfg.mock_trading
    );

    let mut backoff_secs = 1u64;
    let max_backoff_secs = 60u64;
    let mut consecutive_failures = 0u32;
    
}
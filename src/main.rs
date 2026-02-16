/// PM Whale Follower - Main entry point
/// Monitors blockchain for whale trades and executes copy trades

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use alloy::primitives::U256;
use futures::{SinkExt, StreamExt};
use rand::Rng;
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

use polymarket_client_sdk::clob::types::OrderType;
use polymarket_client_sdk::types::Decimal;
use std::sync::Arc;

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

    println!("Hello, world!");
    
}
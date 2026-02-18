// Core types for the trading system

use serde::Deserialize;
use std::fmt;
use std::sync::Arc;
use tokio::sync::oneshot;

/// Parsed order information from blockchain events
#[derive(Debug, Clone)]
pub struct OrderInfo {
    pub order_type: String,
    pub clob_token_id: Arc<str>,  
    pub usd_value: f64,
    pub shares: f64,
    pub price_per_share: f64,
}

/// Fully parsed blockchain event ready for processing
#[derive(Debug, Clone)]
pub struct ParsedEvent {
    pub block_number: u64,
    pub tx_hash: String,
    pub order: OrderInfo,
}

/// Work item for the order processing queue
#[derive(Debug)]
pub struct WorkItem {
    pub event: ParsedEvent,
    pub respond_to: oneshot::Sender<String>,
    pub is_live: Option<bool>,
}

/// Size calculation result 
#[derive(Debug, Clone, Copy)]
pub enum SizeType {
    Scaled,
    ProbHit(u8),   // percentage
    ProbSkip(u8),  // percentage
}

/// Request to resubmit a failed FAK order 
/// Fields ordered to minimize padding: f64s together, then bools/u8 at end
#[derive(Debug, Clone)]
pub struct ResubmitRequest {
    pub token_id: String,       // 24 bytes
    pub whale_price: f64,       // Original whale price
    pub failed_price: f64,      // Price that failed (our limit)
    pub size: f64,              // Order size in shares
    pub whale_shares: f64,      // Whale's trade size (for tier-based max attempts)
    pub max_price: f64,         // Price ceiling (don't exceed this)
    pub cumulative_filled: f64, // Total filled before this attempt
    pub original_size: f64,     // Original order size (for final summary)
    pub side_is_buy: bool,      // Always true for now (only resubmit buys)
    pub is_live: bool,          // Market liveness (for GTD expiry calculation)
    pub attempt: u8,            // Current attempt number (1-indexed)
}

impl fmt::Display for SizeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SizeType::Scaled => f.write_str("SCALED"),
            SizeType::ProbHit(pct) => write!(f, "PROB_HIT ({}%)", pct),
            SizeType::ProbSkip(pct) => write!(f, "PROB_SKIP ({}%)", pct),
        }
    }
}

// ============================================================================
// WebSocket message types (for parsing incoming events)
// ============================================================================

#[derive(Deserialize)]
pub struct WsMessage {
    pub params: Option<WsParams>,
}

#[derive(Deserialize)]
pub struct WsParams {
    pub result: Option<LogResult>,
}

#[derive(Deserialize)]
pub struct LogResult {
    pub topics: Vec<String>,
    pub data: String,
    #[serde(rename = "blockNumber")]
    pub block_number: Option<String>,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<String>,
}
//! Position fetching service for Polymarket data API.

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Rate limiter for API requests
/// Tracks request timestamps to enforce rate limits
struct RateLimiter {
    requests: VecDeque<Instant>,
    max_requests: u32,
    window_duration: Duration,
}

impl RateLimiter {
    fn new(max_requests: u32, window_duration: Duration) -> Self {
        Self {
            requests: VecDeque::with_capacity(max_requests as usize),
            max_requests,
            window_duration,
        }
    }

    /// Wait until we can make a request without exceeding the rate limit
    async fn wait_if_needed(&mut self) {
        let now = Instant::now();

        // Remove requests older than the window
        while let Some(&oldest) = self.requests.front() {
            if now.duration_since(oldest) >= self.window_duration {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // If we've hit the limit, wait until the oldest request expires
        if self.requests.len() >= self.max_requests as usize {
            if let Some(&oldest) = self.requests.front() {
                let wait_time = self
                    .window_duration
                    .saturating_sub(now.duration_since(oldest));
                if !wait_time.is_zero() {
                    tokio::time::sleep(wait_time).await;
                    // After waiting, clean up old requests again
                    let now = Instant::now();
                    while let Some(&oldest) = self.requests.front() {
                        if now.duration_since(oldest) >= self.window_duration {
                            self.requests.pop_front();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        // Record this request
        self.requests.push_back(Instant::now());
    }
}

// Global rate limiter. Lazily initialized to (25 req / 10s) on first use if
// `init_rate_limiter` is not called explicitly at startup.
static RATE_LIMITER: OnceLock<Arc<Mutex<RateLimiter>>> = OnceLock::new();

/// Initialize the global rate limiter with the configured request budget.
///
/// Call this once at startup, before any `fetch_positions_for_wallet` call.
/// Subsequent calls are ignored (the limiter is initialized exactly once).
pub fn init_rate_limiter(max_requests: u32, window_secs: u64) {
    let _ = RATE_LIMITER.set(Arc::new(Mutex::new(RateLimiter::new(
        max_requests,
        Duration::from_secs(window_secs),
    ))));
}

fn rate_limiter() -> &'static Arc<Mutex<RateLimiter>> {
    RATE_LIMITER.get_or_init(|| Arc::new(Mutex::new(RateLimiter::new(25, Duration::from_secs(10)))))
}

/// Fetch current positions for a wallet from Polymarket data API.
///
/// # Arguments
///
/// * `wallet_address` - Ethereum wallet address to query
/// * `api_base_url` - Base URL for the Polymarket data API (e.g., "https://gamma-api.polymarket.com")
///
/// # Returns
///
/// Returns a `Result` containing a vector of position objects, or an error if the request fails.
///
/// # Errors
///
/// Returns an error if:
/// - The HTTP request fails
/// - The response cannot be parsed as JSON
/// - The response status is not successful
pub async fn fetch_positions_for_wallet(
    wallet_address: &str,
    api_base_url: &str,
) -> Result<Vec<HashMap<String, String>>> {
    // Enforce rate limit (configured at startup via init_rate_limiter)
    {
        let mut limiter = rate_limiter().lock().await;
        limiter.wait_if_needed().await;
    }

    let url = format!("{}/positions", api_base_url);

    let client = Client::builder()
        .timeout(Duration::from_secs(30)) // Set default timeout
        .user_agent("Polymarket-Toolkits/1.0") // Set user agent
        .build()?;

    let response = client
        .get(&url)
        .query(&[("user", wallet_address)])
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .context("Failed to send request to positions API")?;

    let response = response
        .error_for_status()
        .context("Positions API returned an error status")?;
    
    // Parse as JSON Value first (handles mixed types: strings, numbers, booleans)
    let positions_json: Vec<serde_json::Value> = response
        .json()
        .await
        .context("Failed to parse positions response as JSON")?;
    
    // Convert each position object to HashMap<String, String>
    // Converting all values (numbers, booleans) to strings
    let positions: Vec<HashMap<String, String>> = positions_json
        .into_iter()
        .filter_map(|pos| {
            pos.as_object().map(|obj| {
                obj.iter()
                    .map(|(k, v)| {
                        let value_str = match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Null => "null".to_string(),
                            _ => v.to_string(), // For arrays/objects, convert to JSON string
                        };
                        (k.clone(), value_str)
                    })
                    .collect()
            })
        })
        .collect();
    
    Ok(positions)
}

// Compare two position snapshots and return a list of inferred order changes.

// Detects: new positions (buy), closed positions (sell), and size changes (buy/sell).

// # Arguments:

// * previous_positions: Positions at time T.
// * current_positions: Positions at time T+1.

// # Returns:

// * List of change dicts with keys: asset, type (BUY/SELL), size, price, slug, etc.

pub async fn detect_position_changes(
    previous_positions: Vec<HashMap<String, String>>,
    current_positions: Vec<HashMap<String, String>>,
) -> Vec<HashMap<String, String>> {
    let previous_by_asset = position_by_asset(previous_positions);
    let current_by_asset = position_by_asset(current_positions);
    let all_asset_ids = previous_by_asset
        .keys()
        .chain(current_by_asset.keys())
        .collect::<HashSet<&String>>();
    let mut changes: Vec<HashMap<String, String>> = Vec::new();

    for asset_id in all_asset_ids {
        let prev_position = previous_by_asset.get(asset_id);
        let curr_position = current_by_asset.get(asset_id);

        if prev_position.is_none() && curr_position.is_some() {
            changes.push(build_new_buy_order(asset_id, curr_position.unwrap()));
            continue;
        }
        if prev_position.is_some() && curr_position.is_none() {
            changes.push(build_full_sell_order(asset_id, prev_position.unwrap()));
            continue;
        }
        if prev_position.is_none() || curr_position.is_none() {
            continue;
        }
        let size_delta = curr_position.unwrap()["size"].parse::<f64>().unwrap()
            - prev_position.unwrap()["size"].parse::<f64>().unwrap();
        if size_delta.abs() < 1e-9 {
            continue;
        }

        changes.push(build_partial_change_order(
            asset_id,
            prev_position.unwrap(),
            curr_position.unwrap(),
            size_delta,
        ))
    }
    return changes;
}

// Build a map of asset id -> position for fast lookup.
fn position_by_asset(
    positions: Vec<HashMap<String, String>>,
) -> HashMap<String, HashMap<String, String>> {
    HashMap::from_iter(
        positions
            .iter()
            .map(|p| (p["asset"].to_string(), p.clone())),
    )
}

// Build a change record for a newly opened position (buy).
fn build_new_buy_order(
    asset_id: &str,
    position: &HashMap<String, String>,
) -> HashMap<String, String> {
    HashMap::from_iter(
        [
            ("asset", asset_id.to_string()),
            ("type", "BUY".to_string()),
            ("size", position["size"].to_string()),
            ("price", position["avgPrice"].to_string()),
            ("title", position["title"].to_string()),
            ("outcome", position["outcome"].to_string()),
            ("conditionId", position["conditionId"].to_string()),
            ("slug", position["slug"].to_string()),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string())),
    )
}

// Build a change record for a fully closed position (sell).
fn build_full_sell_order(
    asset_id: &str,
    position: &HashMap<String, String>,
) -> HashMap<String, String> {
    HashMap::from_iter(
        [
            ("asset", asset_id.to_string()),
            ("type", "SELL".to_string()),
            ("size", position["size"].to_string()),
            ("price", "None".to_string()),
            ("title", position["title"].to_string()),
            ("outcome", position["outcome"].to_string()),
            ("conditionId", position["conditionId"].to_string()),
            ("slug", position["slug"].to_string()),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string())),
    )
}

// Build a change record for a position size increase or decrease.
fn build_partial_change_order(
    asset_id: &str,
    previous_position: &HashMap<String, String>,
    current_position: &HashMap<String, String>,
    size_delta: f64,
) -> HashMap<String, String> {
    let mut base: HashMap<&str, String> = HashMap::from_iter([
        ("asset", asset_id.to_string()),
        ("title", current_position["title"].to_string()),
        ("outcome", current_position["outcome"].to_string()),
        ("conditionId", current_position["conditionId"].to_string()),
        ("slug", current_position["slug"].to_string()),
    ]);
    if size_delta > 0.0 {
        let cost_current = current_position["size"].parse::<f64>().unwrap()
            * current_position["avgPrice"].parse::<f64>().unwrap();
        let cost_previous = previous_position["size"].parse::<f64>().unwrap()
            * previous_position["avgPrice"].parse::<f64>().unwrap();
        let exec_price = (cost_current - cost_previous) / size_delta;
        base.insert("type", "BUY".to_string());
        base.insert("size", size_delta.to_string());
        base.insert("price", exec_price.to_string());
    } else {
        let size_sold = -size_delta;
        let pnl_previous = previous_position
            .get("realizedPnl")
            .unwrap_or(&"0".to_string())
            .parse::<f64>()
            .unwrap();
        let pnl_current = current_position
            .get("realizedPnl")
            .unwrap_or(&"0".to_string())
            .parse::<f64>()
            .unwrap();
        let exec_price = previous_position["avgPrice"].parse::<f64>().unwrap()
            + ((pnl_current - pnl_previous) / size_sold);
        base.insert("type", "SELL".to_string());
        base.insert("size", size_sold.to_string());
        base.insert("price", exec_price.to_string());
    }
    base.into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

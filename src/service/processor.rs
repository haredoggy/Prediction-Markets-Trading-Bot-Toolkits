use alloy::primitives::U256;
use anyhow::anyhow;
use futures::{SinkExt, StreamExt};
use polymarket_client_sdk::{
    auth::{Normal, state::Authenticated},
    clob::{
        Client,
        types::{OrderType, Side},
    },
    types::Decimal,
};
use serde_json::Value;
use std::{cell::RefCell, collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, Utf8Bytes},
};

use crate::{
    OrderArgs, OrderResponse, PreparedCreds,
    config::settings::{
        DEBUG_FULL_ERRORS, FIXED_TRADE_VALUE, MIN_CASH_VALUE, MIN_SHARE_COUNT,
        MIN_WHALE_SHARES_TO_COPY, MONITORED_ADDRESSES, ORDERS_FILLED_EVENT_SIGNATURE,
        RESUBMIT_PRICE_INCREMENT, SCALING_RATIO, TARGET_TOPIC_HEX, USE_PROBABILISTIC_SIZING,
        WS_PING_TIMEOUT, get_gtd_expiry_secs, get_max_resubmit_attempts, get_resubmit_max_buffer,
        get_tier_params, should_increment_price, should_skip_trade,
    },
    models::{OrderInfo, ParsedEvent, ResubmitRequest, SizeType, WorkItem, WsMessage},
    service::{market_cache, orders::{self, OrderEngine}},
    utils::risk_guard::{RiskGuard, RiskGuardConfig, SafetyDecision, calc_liquidity_depth},
};

// ============================================================================
// Thread-local buffers
// ============================================================================

thread_local! {
    static CSV_BUF: RefCell<String> = RefCell::new(String::with_capacity(512));
    static SANITIZE_BUF: RefCell<String> = RefCell::new(String::with_capacity(128));
    static TOKEN_ID_CACHE: RefCell<HashMap<[u8; 32], Arc<str>>> = RefCell::new(HashMap::with_capacity(256));
}

/// Starts a worker thread to process order work items from the queue.
///
/// # Arguments
///
/// * `work_receiver` - Receiver channel for work items
/// * `client` - Shared reference to the CLOB client
/// * `private_key` - Private key for signing orders
/// * `funder_address` - Proxy wallet address (funder)
/// * `enable_trading` - Whether trading is enabled
/// * `mock_trading` - Whether to use mock trading mode
/// * `risk_config` - Configuration for risk guard
/// * `resubmit_sender` - Channel sender for resubmit requests
pub fn start_order_processing_worker(
    work_receiver: mpsc::Receiver<WorkItem>,
    client: Arc<Client<Authenticated<Normal>>>,
    clob_api_base: &str,
    private_key: Arc<String>,
    funder_address: Arc<String>,
    enable_trading: bool,
    mock_trading: bool,
    risk_config: RiskGuardConfig,
    resubmit_sender: mpsc::UnboundedSender<ResubmitRequest>,
) {
    // Convert &str to String so it can be moved into the closure
    let clob_api_base = clob_api_base.to_string();
    std::thread::spawn(move || {
        let mut risk_guard = RiskGuard::new(risk_config);
        process_order_queue(
            work_receiver,
            client,
            &clob_api_base,
            private_key,
            funder_address,
            enable_trading,
            mock_trading,
            &mut risk_guard,
            resubmit_sender,
        );
    });
}

/// Processes order work items from the queue in a blocking loop.
fn process_order_queue(
    mut work_receiver: mpsc::Receiver<WorkItem>,
    client: Arc<Client<Authenticated<Normal>>>,
    clob_api_base: &str,
    private_key: Arc<String>,
    funder_address: Arc<String>,
    enable_trading: bool,
    mock_trading: bool,
    risk_guard: &mut RiskGuard,
    resubmit_sender: mpsc::UnboundedSender<ResubmitRequest>,
) {
    let mut client_mut = (*client).clone();
    while let Some(work_item) = work_receiver.blocking_recv() {
        let status = process_single_order(
            &work_item.event.order,
            &mut client_mut,
            clob_api_base,
            private_key.as_str(),
            funder_address.as_str(),
            enable_trading,
            mock_trading,
            risk_guard,
            &resubmit_sender,
            work_item.is_live,
        );
        let _ = work_item.respond_to.send(status);
    }
}

/// Processes a single order with risk checks and execution.
///
/// # Arguments
///
/// * `order_info` - Information about the order to process
/// * `client` - Mutable reference to the CLOB client
/// * `private_key` - Private key for signing
/// * `funder_address` - Proxy wallet address
/// * `enable_trading` - Whether trading is enabled
/// * `mock_trading` - Whether to use mock trading mode
/// * `risk_guard` - Risk guard instance for safety checks
/// * `resubmit_sender` - Channel sender for resubmit requests
/// * `is_live` - Optional market liveness flag
///
/// # Returns
///
/// A status string describing the result of order processing
fn process_single_order(
    order_info: &OrderInfo,
    client: &mut Client<Authenticated<Normal>>,
    clob_api_base: &str,
    private_key: &str,
    funder_address: &str,
    enable_trading: bool,
    mock_trading: bool,
    risk_guard: &mut RiskGuard,
    resubmit_sender: &mpsc::UnboundedSender<ResubmitRequest>,
    is_live: Option<bool>,
) -> String {
    if !enable_trading {
        return "SKIPPED_DISABLED".into();
    }
    if mock_trading {
        return "MOCK_ONLY".into();
    }

    let is_buy_order = order_info.order_type.starts_with("BUY");
    let whale_share_count = order_info.shares;
    let whale_price_per_share = order_info.price_per_share;

    // Skip small trades to avoid negative expected value after fees
    if should_skip_trade(whale_share_count) {
        return format!("SKIPPED_SMALL (<{:.0} shares)", MIN_WHALE_SHARES_TO_COPY);
    }

    // Risk guard safety check
    let risk_evaluation = risk_guard.check_fast(&order_info.clob_token_id, whale_share_count);
    match risk_evaluation.decision {
        SafetyDecision::Block => {
            return format!("RISK_BLOCKED:{}", risk_evaluation.reason.as_str());
        }
        SafetyDecision::FetchBook => {
            let trade_side = if is_buy_order { Side::Buy } else { Side::Sell };
            match fetch_orderbook_liquidity_depth(
                client,
                clob_api_base,
                &order_info.clob_token_id,
                trade_side,
                whale_price_per_share,
            ) {
                Ok(liquidity_depth) => {
                    let final_evaluation = risk_guard.check_with_book(
                        &order_info.clob_token_id,
                        risk_evaluation.consecutive_large,
                        liquidity_depth,
                    );
                    if final_evaluation.decision == SafetyDecision::Block {
                        return format!("RISK_BLOCKED:{}", final_evaluation.reason.as_str());
                    }
                }
                Err(error) => {
                    risk_guard.trip(&order_info.clob_token_id);
                    return format!("RISK_BOOK_FAIL:{error}");
                }
            }
        }
        SafetyDecision::Allow => {}
    }

    let (price_buffer, order_action_type, size_multiplier) =
        get_tier_params(whale_share_count, is_buy_order, &order_info.clob_token_id);

    // Polymarket valid price range: 0.01 to 0.99 (tick size 0.01)
    let limit_price = if is_buy_order {
        (whale_price_per_share + price_buffer).min(0.99)
    } else {
        (whale_price_per_share - price_buffer).max(0.01)
    };

    let (calculated_share_count, size_calculation_type) =
        calculate_safe_order_size(whale_share_count, limit_price, size_multiplier);
    if calculated_share_count == 0.0 {
        return format!("SKIPPED_PROBABILITY ({})", size_calculation_type);
    }

    // Use SDK for order placement (handles authentication automatically)
    let token_id = order_info.clob_token_id.to_string();
    // For market orders (FAK): size max 4 decimals, for limit orders: size max 2 decimals
    let rounded_size = if order_action_type == "FAK" {
        (calculated_share_count * 10000.0).floor() / 10000.0 // Round to 4 decimals
    } else {
        (calculated_share_count * 100.0).floor() / 100.0 // Round to 2 decimals
    };
    let size_decimal = match Decimal::try_from(rounded_size) {
        Ok(decimal) => decimal,
        Err(error) => return format!("EXEC_FAIL: Invalid size: {}", error),
    };
    let price_decimal = match Decimal::try_from(limit_price) {
        Ok(decimal) => decimal,
        Err(error) => return format!("EXEC_FAIL: Invalid price: {}", error),
    };

    let sdk_order_type = if order_action_type == "FAK" {
        OrderType::FOK
    } else if order_action_type == "GTD" {
        OrderType::GTD
    } else {
        OrderType::GTC
    };

    // Run async SDK function from blocking context
    // Create a minimal runtime if we're in a blocking thread without one
    let private_key_clone = private_key.to_string(); // Clone for async move
    let funder_address_clone = funder_address.to_string(); // Clone for async move
    let token_id_clone = token_id.clone();
    let async_order_task = async move {
        if is_buy_order && order_action_type == "FAK" {
            // Market buy order using USDC amount
            // For market orders: maker (USDC) max 2 decimals, taker (shares) max 4 decimals
            // Round USDC to 2 decimals, size is already rounded to 4 decimals above
            let usdc_amount = (size_decimal * price_decimal).round_dp(2);

            // Polymarket requires minimum $1 USDC for market buy orders
            // Use MAX(MIN_CASH_VALUE, 1.0) to respect user config but enforce Polymarket minimum
            let min_usdc = MIN_CASH_VALUE.max(1.0);
            let min_usdc_decimal =
                Decimal::try_from(min_usdc).unwrap_or_else(|_| Decimal::from(1u64)); // Fallback to $1.00 if conversion fails
            if usdc_amount < min_usdc_decimal {
                return Err(anyhow!(
                    "Market buy order amount ${} is below minimum ${} USDC (Polymarket minimum: $1.00)",
                    usdc_amount,
                    min_usdc
                ));
            }

            orders::buy_order(
                &private_key_clone,
                &funder_address_clone,
                &token_id_clone,
                usdc_amount,
                Some(sdk_order_type),
            )
            .await
        } else if is_buy_order {
            // Limit buy order
            orders::buy_limit_order(
                &private_key_clone,
                &funder_address_clone,
                &token_id_clone,
                size_decimal,
                price_decimal,
                Some(sdk_order_type),
            )
            .await
        } else {
            // Sell order
            orders::sell_order(
                &private_key_clone,
                &funder_address_clone,
                &token_id_clone,
                size_decimal,
                price_decimal,
                Some(sdk_order_type),
            )
            .await
        }
    };

    let order_result = match tokio::runtime::Handle::try_current() {
        Ok(runtime_handle) => {
            // We're in a Tokio context, use the current runtime
            runtime_handle.block_on(async_order_task)
        }
        Err(_) => {
            // We're in a blocking thread without a runtime, create one
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async_order_task)
        }
    };

    match order_result {
        Ok(order_response) => {
            // SDK returns PostOrderResponse - extract filled amounts
            // Check if order was successful (no error message means success)
            let http_status_code = if order_response.error_msg.is_none()
                || order_response.error_msg.as_ref().unwrap().is_empty()
            {
                200
            } else {
                400
            };

            let status_string = format!("{:?}", order_response.status);
            let response_body = if order_response.error_msg.is_some() {
                order_response.error_msg.as_ref().unwrap().clone()
            } else {
                format!("{:?}", order_response)
            };

            let parsed_order_response: Option<OrderResponse> = if http_status_code == 200 {
                // Convert SDK response to our OrderResponse format
                Some(OrderResponse {
                    success: true,
                    error_msg: order_response.error_msg.clone().unwrap_or_default(),
                    order_id: order_response.order_id.to_string(),
                    transactions_hashes: order_response
                        .transaction_hashes
                        .iter()
                        .map(|hash| format!("{:?}", hash))
                        .collect(),
                    status: status_string,
                    taking_amount: order_response.taking_amount.to_string(),
                    making_amount: order_response.making_amount.to_string(),
                })
            } else {
                None
            };

            let mut underfill_message: Option<String> = None;
            if let Some(ref response) = parsed_order_response {
                if is_buy_order && order_action_type == "FAK" {
                    let filled_share_count: f64 = response.taking_amount.parse().unwrap_or(0.0);
                    let requested_share_count = (calculated_share_count * 100.0).floor() / 100.0;

                    if filled_share_count < requested_share_count && filled_share_count > 0.0 {
                        let remaining_share_count = requested_share_count - filled_share_count;

                        let minimum_threshold = MIN_SHARE_COUNT.max(MIN_CASH_VALUE / limit_price);
                        if remaining_share_count >= minimum_threshold {
                            let resubmit_price_buffer = get_resubmit_max_buffer(whale_share_count);
                            let maximum_resubmit_price =
                                (limit_price + resubmit_price_buffer).min(0.99);
                            let resubmit_request = ResubmitRequest {
                                token_id: order_info.clob_token_id.to_string(),
                                whale_price: whale_price_per_share,
                                failed_price: limit_price, // Start at same price (already filled some)
                                size: (remaining_share_count * 100.0).floor() / 100.0,
                                whale_shares: whale_share_count,
                                side_is_buy: true,
                                attempt: 1,
                                max_price: maximum_resubmit_price,
                                cumulative_filled: filled_share_count,
                                original_size: requested_share_count,
                                is_live: is_live.unwrap_or(false),
                            };
                            let _ = resubmit_sender.send(resubmit_request);
                            underfill_message = Some(format!(
                                " | \x1b[33mUNDERFILL: {:.2}/{:.2} filled, resubmit {:.2}\x1b[0m",
                                filled_share_count, calculated_share_count, remaining_share_count
                            ));
                        }
                    }
                }
            }

            if http_status_code == 400 && response_body.contains("FAK") && is_buy_order {
                let resubmit_price_buffer = get_resubmit_max_buffer(whale_share_count);
                let maximum_resubmit_price = (limit_price + resubmit_price_buffer).min(0.99);
                let rounded_order_size = (calculated_share_count * 100.0).floor() / 100.0;
                let resubmit_request = ResubmitRequest {
                    token_id: order_info.clob_token_id.to_string(),
                    whale_price: whale_price_per_share,
                    failed_price: limit_price,
                    size: rounded_order_size,
                    whale_shares: whale_share_count,
                    side_is_buy: true,
                    attempt: 1,
                    max_price: maximum_resubmit_price,
                    cumulative_filled: 0.0,
                    original_size: rounded_order_size,
                    is_live: is_live.unwrap_or(false),
                };
                let _ = resubmit_sender.send(resubmit_request);
            }

            // Extract filled shares and actual fill price for display (reuse parsed response)
            let (filled_share_count, actual_fill_price) = parsed_order_response
                .as_ref()
                .and_then(|response| {
                    let taking_amount: f64 = response.taking_amount.parse().ok()?;
                    let making_amount: f64 = response.making_amount.parse().ok()?;
                    if taking_amount > 0.0 {
                        Some((taking_amount, making_amount / taking_amount))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| {
                    if http_status_code == 200 {
                        (calculated_share_count, limit_price)
                    } else {
                        (0.0, limit_price)
                    }
                });

            // Format with color-coded fill percentage
            const PINK_COLOR: &str = "\x1b[38;5;199m";
            const RESET_COLOR: &str = "\x1b[0m";
            let fill_percentage_color = get_fill_color(filled_share_count, calculated_share_count);
            let whale_size_color = get_whale_size_color(whale_share_count);
            let status_display = if http_status_code == 200 {
                "200 OK"
            } else {
                "FAILED"
            };
            let mut status_message = format!(
                "{} [{}] | {}{:.2}/{:.2}{} filled @ {}{:.2}{} | {}whale {:.1}{} @ {:.2}",
                status_display,
                size_calculation_type,
                fill_percentage_color,
                filled_share_count,
                calculated_share_count,
                RESET_COLOR,
                PINK_COLOR,
                actual_fill_price,
                RESET_COLOR,
                whale_size_color,
                whale_share_count,
                RESET_COLOR,
                whale_price_per_share
            );
            if let Some(message) = underfill_message {
                status_message.push_str(&message);
            }
            if http_status_code != 200 {
                // Provide helpful guidance for common errors
                if response_body.contains("not enough balance")
                    || response_body.contains("allowance")
                {
                    status_message.push_str(&format!(" | \x1b[33m⚠️  INSUFFICIENT BALANCE/ALLOWANCE - Fix: 1) Check USDC balance on Polygon 2) Approve exchange at https://polymarket.com (make a test trade)\x1b[0m"));
                } else {
                    status_message.push_str(&format!(" | {}", response_body));
                }
            }
            status_message
        }
        Err(error) => {
            let error_message = error.to_string();

            // If error is already formatted with INSUFFICIENT_BALANCE/ALLOWANCE, use it as-is
            if error_message.contains("INSUFFICIENT_BALANCE/ALLOWANCE")
                || error_message.contains("Insufficient balance/allowance")
            {
                // Error already formatted by orders.rs, use it directly
                format!("EXEC_FAIL: {}", error_message)
            } else {
                // Check error chain for balance/allowance issues
                let mut error_chain_messages: Vec<String> = Vec::new();
                for chain_error in error.chain() {
                    let chain_message = chain_error.to_string();
                    error_chain_messages.push(chain_message.clone());
                    if chain_message.contains("not enough balance")
                        || chain_message.contains("allowance")
                        || chain_message.contains("INSUFFICIENT")
                    {
                        // Found balance/allowance error in chain
                        return format!(
                            "EXEC_FAIL: INSUFFICIENT_BALANCE/ALLOWANCE - {} | Fix: 1) Deposit USDC to your Polygon wallet 2) Approve USDC spending at https://polymarket.com (make a test trade)",
                            chain_message
                        );
                    }
                }

                // Check main error message
                if error_message.contains("not enough balance")
                    || error_message.contains("allowance")
                {
                    format!(
                        "EXEC_FAIL: INSUFFICIENT_BALANCE/ALLOWANCE | Fix: 1) Deposit USDC to your Polygon wallet 2) Approve USDC spending at https://polymarket.com (make a test trade) 3) Ensure wallet has approved exchange contract"
                    )
                } else {
                    format!(
                        "EXEC_FAIL: {} | chain: {}",
                        error_message,
                        error_chain_messages.join(" -> ")
                    )
                }
            }
        }
    }
}

/// Fetches orderbook liquidity depth for a given token and trade side.
///
/// This is a blocking function that runs async operations in a blocking context.
///
/// # Arguments
///
/// * `client` - Reference to the CLOB client
/// * `token_id` - The CLOB token ID
/// * `trade_side` - Buy or sell side
/// * `price_threshold` - Price threshold for depth calculation
///
/// # Returns
///
/// Liquidity depth value or an error string
fn fetch_orderbook_liquidity_depth(
    client: &Client<Authenticated<Normal>>,
    clob_api_base: &str,
    token_id: &str,
    trade_side: Side,
    price_threshold: f64,
) -> Result<f64, &'static str> {
    let url = format!("{}/book?token_id={}", clob_api_base, token_id);
    let handle = tokio::runtime::Handle::current();
    // let resp = handle
    //     .block_on(
    //         client
    //             .http_client()
    //             .get(&url)
    //             .timeout(Duration::from_millis(500))
    //             .send(),
    //     )
    //     .map_err(|_| "NETWORK")?;

    // if !resp.status().is_success() {
    //     return Err("HTTP_ERROR");
    // }

    // let orderbook_data: Value = handle.block_on(resp.json()).map_err(|_| "PARSE")?;
    // let orderbook_side_key = if trade_side == Side::Buy {
    //     "asks"
    // } else {
    //     "bids"
    // };

    // Stack array instead of Vec - avoids heap allocation for max 10 items
    let mut price_levels: [(f64, f64); 10] = [(0.0, 0.0); 10];
    let mut level_count = 0;
    // if let Some(levels_array) = orderbook_data[orderbook_side_key].as_array() {
    //     for level in levels_array.iter().take(10) {
    //         if let (Some(price), Some(size)) = (
    //             level["price"].as_str().and_then(|s| s.parse().ok()),
    //             level["size"].as_str().and_then(|s| s.parse().ok()),
    //         ) {
    //             price_levels[level_count] = (price, size);
    //             level_count += 1;
    //         }
    //     }
    // }

    Ok(calc_liquidity_depth(
        trade_side,
        &price_levels[..level_count],
        price_threshold,
    ))
}

/// Calculates a safe order size based on whale trade size, price, and multiplier.
///
/// Uses either fixed trade value mode or scaling-based logic with optional probabilistic sizing.
///
/// # Arguments
///
/// * `whale_share_count` - The number of shares in the whale trade
/// * `price_per_share` - Price per share
/// * `size_multiplier` - Multiplier for size calculation
///
/// # Returns
///
/// Tuple of (calculated_size, size_type)
fn calculate_safe_order_size(
    whale_share_count: f64,
    price_per_share: f64,
    size_multiplier: f64,
) -> (f64, SizeType) {
    // Fixed $1 per trade mode (tx tracker wallet style)
    if FIXED_TRADE_VALUE > 0.0 {
        let safe_price = price_per_share.max(0.0001);
        let fixed_share_count = FIXED_TRADE_VALUE / safe_price;
        // Ensure minimum Polymarket requirement ($1.01)
        let minimum_share_count = (1.01 / safe_price).max(MIN_SHARE_COUNT);
        let final_share_count = fixed_share_count.max(minimum_share_count);
        return (final_share_count, SizeType::Scaled);
    }

    // Original scaling-based logic
    let target_scaled_size = whale_share_count * SCALING_RATIO * size_multiplier;
    let safe_price = price_per_share.max(0.0001);
    let required_minimum_size = (MIN_CASH_VALUE / safe_price).max(MIN_SHARE_COUNT);

    if target_scaled_size >= required_minimum_size {
        return (target_scaled_size, SizeType::Scaled);
    }

    if !USE_PROBABILISTIC_SIZING {
        return (required_minimum_size, SizeType::Scaled);
    }

    let fill_probability = target_scaled_size / required_minimum_size;
    let probability_percentage = (fill_probability * 100.0) as u8;
    if rand::random::<f64>() < fill_probability {
        (
            required_minimum_size,
            SizeType::ProbHit(probability_percentage),
        )
    } else {
        (0.0, SizeType::ProbSkip(probability_percentage))
    }
}

/// Returns ANSI color code based on fill percentage.
///
/// # Arguments
///
/// * `filled_amount` - Amount filled
/// * `requested_amount` - Amount requested
///
/// # Returns
///
/// ANSI color escape code string
fn get_fill_color(filled_amount: f64, requested_amount: f64) -> &'static str {
    if requested_amount <= 0.0 {
        return "\x1b[31m"; // Red if no request
    }
    let fill_percentage = (filled_amount / requested_amount) * 100.0;
    if fill_percentage < 50.0 {
        "\x1b[31m" // Red
    } else if fill_percentage < 75.0 {
        "\x1b[38;5;208m" // Orange
    } else if fill_percentage < 90.0 {
        "\x1b[33m" // Yellow
    } else {
        "\x1b[32m" // Green
    }
}

/// Returns ANSI color code based on whale share count (gradient from small to large).
///
/// # Arguments
///
/// * `share_count` - Number of shares
///
/// # Returns
///
/// ANSI color escape code string
fn get_whale_size_color(share_count: f64) -> &'static str {
    if share_count < 500.0 {
        "\x1b[90m" // Gray (very small)
    } else if share_count < 1000.0 {
        "\x1b[36m" // Cyan (small)
    } else if share_count < 2000.0 {
        "\x1b[34m" // Blue (medium-small)
    } else if share_count < 5000.0 {
        "\x1b[32m" // Green (medium)
    } else if share_count < 8000.0 {
        "\x1b[33m" // Yellow (medium-large)
    } else if share_count < 15000.0 {
        "\x1b[38;5;208m" // Orange (large)
    } else {
        "\x1b[35m" // Magenta (huge)
    }
}

pub async fn run_ws_loop(
    order_engine: &OrderEngine,
    gamma_api_base: &str,
    clob_api_base: &str,
    clob_wss_url: &str,
) -> Result<(), anyhow::Error> {
    // Add connection timeout to prevent hanging on TLS handshake
    let (mut ws, _) = tokio::time::timeout(Duration::from_secs(10), connect_async(clob_wss_url))
        .await
        .map_err(|_| anyhow!("Connection timeout"))??;

    let sub = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "eth_subscribe",
        "params": ["logs", {
            "address": MONITORED_ADDRESSES,
            "topics": [[ORDERS_FILLED_EVENT_SIGNATURE], Value::Null, TARGET_TOPIC_HEX.as_str()]
        }]
    })
    .to_string();

    println!("🔌 Connected. Subscribing...");
    ws.send(Message::Text(Utf8Bytes::from(sub))).await?;

    let http_client = reqwest::Client::builder().no_proxy().build()?;

    // Convert &str to String once before the loop so they can be moved into async closures
    let gamma_api_base_owned = gamma_api_base.to_string();
    let clob_api_base_owned = clob_api_base.to_string();

    loop {
        let msg = tokio::time::timeout(WS_PING_TIMEOUT, ws.next())
            .await
            .map_err(|_| anyhow!("WS timeout"))?
            .ok_or_else(|| anyhow!("WS closed"))??;

        match msg {
            Message::Text(text) => {
                if let Some(evt) = parse_event(text.to_string()) {
                    let engine = order_engine.clone();
                    let client = http_client.clone();
                    let gamma_api_base_clone = gamma_api_base_owned.clone();
                    let clob_api_base_clone = clob_api_base_owned.clone();
                    tokio::spawn(async move {
                        handle_event(
                            evt,
                            &engine,
                            &client,
                            &gamma_api_base_clone,
                            &clob_api_base_clone,
                        )
                        .await
                    });
                }
            }
            Message::Binary(bin) => {
                if let Ok(text) = String::from_utf8(bin.to_vec()) {
                    if let Some(evt) = parse_event(text) {
                        let engine = order_engine.clone();
                        let client = http_client.clone();
                        let gamma_api_base_clone = gamma_api_base_owned.clone();
                        let clob_api_base_clone = clob_api_base_owned.clone();
                        tokio::spawn(async move {
                            handle_event(
                                evt,
                                &engine,
                                &client,
                                &gamma_api_base_clone,
                                &clob_api_base_clone,
                            )
                            .await
                        });
                    }
                }
            }
            Message::Ping(d) => {
                ws.send(Message::Pong(d)).await?;
            }
            Message::Close(f) => return Err(anyhow!("WS closed: {:?}", f)),
            _ => {}
        }
    }
}

async fn handle_event(
    evt: ParsedEvent,
    order_engine: &OrderEngine,
    http_client: &reqwest::Client,
    gamma_api_base: &str,
    clob_api_base: &str,
) {
    // Check live status from cache, fallback to API lookup
    let is_live = match market_cache::get_is_live(&evt.order.clob_token_id) {
        Some(v) => Some(v),
        None => fetch_is_live(&evt.order.clob_token_id, http_client, gamma_api_base).await,
    };

    let status = order_engine.submit(evt.clone(), is_live).await;

    tokio::time::sleep(Duration::from_secs_f32(2.8)).await;

    // Fetch order book for post-trade logging
    let bests = fetch_best_book(
        &evt.order.clob_token_id,
        &evt.order.order_type,
        http_client,
        clob_api_base,
    )
    .await;
    let ((bp, bs), (sp, ss)) =
        bests.unwrap_or_else(|| (("N/A".into(), "N/A".into()), ("N/A".into(), "N/A".into())));
    let is_live = is_live.unwrap_or(false);

    // Highlight best price in bright pink
    let pink = "\x1b[38;5;199m";
    let reset = "\x1b[0m";
    let colored_bp = format!("{}{}{}", pink, bp, reset);

    let live_display = if is_live {
        format!("\x1b[34mlive: true\x1b[0m")
    } else {
        "live: false".to_string()
    };

    println!(
        "⚡ [B:{}] {} | ${:.0} | {} | best: {} @ {} | 2nd: {} @ {} | {}",
        evt.block_number,
        evt.order.order_type,
        evt.order.usd_value,
        status,
        colored_bp,
        bs,
        sp,
        ss,
        live_display
    );
}

// ============================================================================
// Resubmitter Worker (handles FAK failures with price escalation)
// ============================================================================

pub async fn resubmit_worker(
    mut rx: mpsc::UnboundedReceiver<ResubmitRequest>,
    client: Arc<Client<Authenticated<Normal>>>,
    creds: Arc<PreparedCreds>,
) {
    println!("🔄 Resubmitter worker started");

    while let Some(req) = rx.recv().await {
        let max_attempts = get_max_resubmit_attempts(req.whale_shares);
        let is_last_attempt = req.attempt >= max_attempts;

        // Calculate increment: chase only if should_increment_price returns true
        let increment = if should_increment_price(req.whale_shares, req.attempt) {
            RESUBMIT_PRICE_INCREMENT
        } else {
            0.0 // Flat retry
        };
        let new_price = if req.side_is_buy {
            (req.failed_price + increment).min(0.99)
        } else {
            (req.failed_price - increment).max(0.01)
        };

        // Check if we've exceeded max buffer (skip check for GTD - last attempt always goes through)
        if !is_last_attempt && req.side_is_buy && new_price > req.max_price {
            let fill_pct = if req.original_size > 0.0 {
                (req.cumulative_filled / req.original_size) * 100.0
            } else {
                0.0
            };
            println!(
                "🔄 Resubmit ABORT: attempt {} price {:.2} > max {:.2} | filled {:.2}/{:.2} ({:.0}%)",
                req.attempt,
                new_price,
                req.max_price,
                req.cumulative_filled,
                req.original_size,
                fill_pct
            );
            continue;
        }

        let client_clone = Arc::clone(&client);
        let creds_clone = Arc::clone(&creds);
        let token_id = req.token_id.clone();
        let size = req.size;
        let attempt = req.attempt;
        let whale_price = req.whale_price;
        let max_price = req.max_price;
        let is_live = req.is_live;

        // Submit order: FAK for early attempts, GTD with expiry for last attempt
        let result = submit_resubmit_order_sync(
            &client_clone,
            &creds_clone,
            &token_id,
            new_price,
            size,
            is_live,
            is_last_attempt,
        )
        .await;

        match result {
            Ok((true, _, filled_this_attempt)) => {
                if is_last_attempt {
                    // GTD order placed on book - we don't know fill amount yet
                    println!(
                        "\x1b[32m🔄 Resubmit GTD SUBMITTED: attempt {} @ {:.2} | size {:.2} | prior filled {:.2}/{:.2}\x1b[0m",
                        attempt, new_price, size, req.cumulative_filled, req.original_size
                    );
                } else {
                    // FAK order - check if partial fill
                    let total_filled = req.cumulative_filled + filled_this_attempt;
                    let fill_pct = if req.original_size > 0.0 {
                        (total_filled / req.original_size) * 100.0
                    } else {
                        0.0
                    };
                    let remaining = size - filled_this_attempt;

                    // If partial fill, continue with remaining size
                    if remaining > 1.0 && filled_this_attempt > 0.0 {
                        println!(
                            "\x1b[33m🔄 Resubmit PARTIAL: attempt {} @ {:.2} | filled {:.2}/{:.2} ({:.0}%) | remaining {:.2}\x1b[0m",
                            attempt,
                            new_price,
                            total_filled,
                            req.original_size,
                            fill_pct,
                            remaining
                        );
                        let next_req = ResubmitRequest {
                            token_id: req.token_id,
                            whale_price,
                            failed_price: new_price,
                            size: remaining,
                            whale_shares: req.whale_shares,
                            side_is_buy: req.side_is_buy,
                            attempt: attempt + 1,
                            max_price,
                            cumulative_filled: total_filled,
                            original_size: req.original_size,
                            is_live: req.is_live,
                        };
                        let _ = process_resubmit_chain(&client, &creds, next_req).await;
                    } else {
                        println!(
                            "\x1b[32m🔄 Resubmit SUCCESS: attempt {} @ {:.2} | filled {:.2}/{:.2} ({:.0}%)\x1b[0m",
                            attempt, new_price, total_filled, req.original_size, fill_pct
                        );
                    }
                }
            }
            Ok((false, body, filled_this_attempt)) => {
                if attempt < max_attempts {
                    // Re-queue with updated price
                    let next_req = ResubmitRequest {
                        token_id: req.token_id,
                        whale_price,
                        failed_price: new_price,
                        size: req.size,
                        whale_shares: req.whale_shares,
                        side_is_buy: req.side_is_buy,
                        attempt: attempt + 1,
                        max_price,
                        cumulative_filled: req.cumulative_filled + filled_this_attempt,
                        original_size: req.original_size,
                        is_live: req.is_live,
                    };
                    let next_increment = if should_increment_price(req.whale_shares, attempt + 1) {
                        RESUBMIT_PRICE_INCREMENT
                    } else {
                        0.0
                    };
                    println!(
                        "🔄 Resubmit attempt {} failed (FAK), retrying @ {:.2} (max: {})",
                        attempt,
                        new_price + next_increment,
                        max_attempts
                    );
                    if req.whale_shares < 1000.0 {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                    let _ = process_resubmit_chain(&client, &creds, next_req).await;
                } else {
                    let total_filled = req.cumulative_filled + filled_this_attempt;
                    let fill_pct = if req.original_size > 0.0 {
                        (total_filled / req.original_size) * 100.0
                    } else {
                        0.0
                    };
                    let error_msg = if DEBUG_FULL_ERRORS {
                        body.clone()
                    } else {
                        body.chars().take(80).collect::<String>()
                    };
                    println!(
                        "🔄 Resubmit FAILED: attempt {} @ {:.2} | filled {:.2}/{:.2} ({:.0}%) | {}",
                        attempt, new_price, total_filled, req.original_size, fill_pct, error_msg
                    );
                }
            }
            Err(e) => {
                let fill_pct = if req.original_size > 0.0 {
                    (req.cumulative_filled / req.original_size) * 100.0
                } else {
                    0.0
                };
                println!(
                    "🔄 Resubmit ERROR: attempt {} | filled {:.2}/{:.2} ({:.0}%) | {}",
                    attempt, req.cumulative_filled, req.original_size, fill_pct, e
                );
            }
        }
    }
}

async fn process_resubmit_chain(
    client: &Arc<Client<Authenticated<Normal>>>,
    creds: &Arc<PreparedCreds>,
    mut req: ResubmitRequest,
) {
    let max_attempts = get_max_resubmit_attempts(req.whale_shares);

    while req.attempt <= max_attempts {
        let is_last_attempt = req.attempt >= max_attempts;

        // Calculate increment: chase only if should_increment_price returns true
        let increment = if should_increment_price(req.whale_shares, req.attempt) {
            RESUBMIT_PRICE_INCREMENT
        } else {
            0.0 // Flat retry
        };
        let new_price = if req.side_is_buy {
            (req.failed_price + increment).min(0.99)
        } else {
            (req.failed_price - increment).max(0.01)
        };

        // Check if we've exceeded max buffer (skip check for GTD - last attempt always goes through)
        if !is_last_attempt && req.side_is_buy && new_price > req.max_price {
            let fill_pct = if req.original_size > 0.0 {
                (req.cumulative_filled / req.original_size) * 100.0
            } else {
                0.0
            };
            println!(
                "🔄 Resubmit chain ABORT: attempt {} price {:.2} > max {:.2} | filled {:.2}/{:.2} ({:.0}%)",
                req.attempt,
                new_price,
                req.max_price,
                req.cumulative_filled,
                req.original_size,
                fill_pct
            );
            return;
        }

        let client_clone = Arc::clone(&client);
        let creds_clone = Arc::clone(&creds);
        let token_id = req.token_id.clone();
        let size = req.size;
        let attempt = req.attempt;
        let is_live = req.is_live;

        // Submit order: FAK for early attempts, GTD with expiry for last attempt
        let result = submit_resubmit_order_sync(
            &client_clone,
            &creds_clone,
            &token_id,
            new_price,
            size,
            is_live,
            is_last_attempt,
        )
        .await;

        match result {
            Ok((true, _, filled_this_attempt)) => {
                if is_last_attempt {
                    // GTD order placed on book - we don't know fill amount yet
                    println!(
                        "\x1b[32m🔄 Resubmit chain GTD SUBMITTED: attempt {} @ {:.2} | size {:.2} | prior filled {:.2}/{:.2}\x1b[0m",
                        attempt, new_price, req.size, req.cumulative_filled, req.original_size
                    );
                    return;
                } else {
                    // FAK order - check if partial fill
                    let total_filled = req.cumulative_filled + filled_this_attempt;
                    let fill_pct = if req.original_size > 0.0 {
                        (total_filled / req.original_size) * 100.0
                    } else {
                        0.0
                    };
                    let remaining = req.size - filled_this_attempt;

                    // If partial fill, continue with remaining size
                    if remaining > 1.0 && filled_this_attempt > 0.0 {
                        println!(
                            "\x1b[33m🔄 Resubmit chain PARTIAL: attempt {} @ {:.2} | filled {:.2}/{:.2} ({:.0}%) | remaining {:.2}\x1b[0m",
                            attempt,
                            new_price,
                            total_filled,
                            req.original_size,
                            fill_pct,
                            remaining
                        );
                        req.cumulative_filled = total_filled;
                        req.size = remaining;
                        req.failed_price = new_price;
                        req.attempt += 1;
                        continue;
                    } else {
                        println!(
                            "\x1b[32m🔄 Resubmit chain SUCCESS: attempt {} @ {:.2} | filled {:.2}/{:.2} ({:.0}%)\x1b[0m",
                            attempt, new_price, total_filled, req.original_size, fill_pct
                        );
                        return;
                    }
                }
            }
            Ok((false, body, filled_this_attempt))
                if body.contains("FAK") && attempt < max_attempts =>
            {
                req.cumulative_filled += filled_this_attempt;
                req.failed_price = new_price;
                req.attempt += 1;
                // Small trades get 50ms delay to let orderbook refresh
                if req.whale_shares < 1000.0 {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                continue;
            }
            Ok((false, body, filled_this_attempt)) => {
                let total_filled = req.cumulative_filled + filled_this_attempt;
                let fill_pct = if req.original_size > 0.0 {
                    (total_filled / req.original_size) * 100.0
                } else {
                    0.0
                };
                let fill_color = get_fill_color(total_filled, req.original_size);
                let reset = "\x1b[0m";
                let error_msg = if DEBUG_FULL_ERRORS {
                    body.clone()
                } else {
                    body.chars().take(80).collect::<String>()
                };
                println!(
                    "🔄 Resubmit chain FAILED: attempt {}/{} @ {:.2} | {}filled {:.2}/{:.2} ({:.0}%){} | {}",
                    attempt,
                    max_attempts,
                    new_price,
                    fill_color,
                    total_filled,
                    req.original_size,
                    fill_pct,
                    reset,
                    error_msg
                );
                return;
            }
            Err(e) => {
                let fill_pct = if req.original_size > 0.0 {
                    (req.cumulative_filled / req.original_size) * 100.0
                } else {
                    0.0
                };
                let fill_color = get_fill_color(req.cumulative_filled, req.original_size);
                let reset = "\x1b[0m";
                println!(
                    "🔄 Resubmit chain ERROR: attempt {} | {}filled {:.2}/{:.2} ({:.0}%){} | {}",
                    attempt,
                    fill_color,
                    req.cumulative_filled,
                    req.original_size,
                    fill_pct,
                    reset,
                    e
                );
                return;
            }
            Err(e) => {
                let fill_pct = if req.original_size > 0.0 {
                    (req.cumulative_filled / req.original_size) * 100.0
                } else {
                    0.0
                };
                let fill_color = get_fill_color(req.cumulative_filled, req.original_size);
                let reset = "\x1b[0m";
                println!(
                    "🔄 Resubmit chain TASK ERROR: {}filled {:.2}/{:.2} ({:.0}%){} | {}",
                    fill_color, req.cumulative_filled, req.original_size, fill_pct, reset, e
                );
                return;
            }
        }
    }
}

/// Returns (success, body_text, filled_shares)
async fn submit_resubmit_order_sync(
    client: &Client<Authenticated<Normal>>,
    creds: &PreparedCreds,
    token_id: &str,
    price: f64,
    size: f64,
    is_live: bool,
    is_last_attempt: bool,
) -> anyhow::Result<(bool, String, f64)> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut client = client.clone();

    // Only use GTD with expiry on the LAST attempt; earlier attempts use FAK
    let (expiration, order_type) = if is_last_attempt {
        let expiry_secs = get_gtd_expiry_secs(is_live);
        let expiry_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + expiry_secs;
        (Some(expiry_timestamp.to_string()), "GTD")
    } else {
        (None, "FAK")
    };

    // Round to micro-units (6 decimals) then back to avoid floating-point truncation issues
    // e.g., 40.80 stored as 40.7999999... would truncate to 40799999 instead of 40800000
    let price_micro = (price * 1_000_000.0).round() as i64;
    let size_micro = (size * 1_000_000.0).round() as i64;
    let rounded_price = price_micro as f64 / 1_000_000.0;
    let rounded_size = size_micro as f64 / 1_000_000.0;

    let args = OrderArgs {
        token_id: token_id.to_string(),
        price: rounded_price,
        size: rounded_size,
        side: "BUY".into(),
        fee_rate_bps: None,
        nonce: Some(0),
        expiration,
        taker: None,
        order_type: Some(order_type.to_string()),
    };

    // let signed = client.create_order(args).await?;
    // let body = signed.post_body(&creds.api_key, order_type);
    // let resp = client.post_order_fast(body, creds).await?;

    // let status = resp.status();
    // let body_text = resp.text().await.unwrap_or_default();

    // Parse filled amount from successful responses
    // GTD orders return taking_amount=0 since they're placed on book, not immediately filled
    // For GTD, return 0 - caller handles GTD success messaging separately
    // let filled_shares = if status.is_success() && order_type == "FAK" {
    //     serde_json::from_str::<OrderResponse>(&body_text)
    //         .ok()
    //         .and_then(|r| r.taking_amount.parse::<f64>().ok())
    //         .unwrap_or(0.0)
    // } else {
    //     0.0
    // };

    // Ok((status.is_success(), body_text.to_owned(), filled_shares))
    Ok((true, "".to_owned(), 0.0))
}

async fn fetch_is_live(
    token_id: &str,
    client: &reqwest::Client,
    gamma_api_base: &str,
) -> Option<bool> {
    // Fetch market info to get slug
    let market_url = format!("{}/markets?clob_token_ids={}", gamma_api_base, token_id);
    let resp = client
        .get(&market_url)
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .ok()?;
    let val: Value = resp.json().await.ok()?;
    let slug = val.get(0)?.get("slug")?.as_str()?.to_string();

    // Fetch live status from events API
    let event_url = format!("{}/events/slug/{}", gamma_api_base, slug);
    let resp = client
        .get(&event_url)
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .ok()?;
    let val: Value = resp.json().await.ok()?;

    Some(val["live"].as_bool().unwrap_or(false))
}

async fn fetch_best_book(
    token_id: &str,
    order_type: &str,
    client: &reqwest::Client,
    clob_api_base: &str,
) -> Option<((String, String), (String, String))> {
    let url = format!("{}/book?token_id={}", clob_api_base, token_id);
    let resp = client
        .get(&url)
        .timeout(Duration::from_millis(2500)) // TODO: Use config.trading.book_req_timeout()
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let val: Value = resp.json().await.ok()?;
    let key = if order_type.starts_with("BUY") {
        "asks"
    } else {
        "bids"
    };
    let entries = val.get(key)?.as_array()?;

    let is_buy = order_type.starts_with("BUY");

    let (best, second): (Option<(&Value, f64)>, Option<(&Value, f64)>) =
        entries.iter().fold((None, None), |(best, second), entry| {
            let price: f64 = match entry
                .get("price")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
            {
                Some(p) => p,
                None => return (best, second),
            };

            let better = |candidate: f64, current: f64| {
                if is_buy {
                    candidate < current
                } else {
                    candidate > current
                }
            };

            match best {
                Some((_, bp)) if better(price, bp) => (Some((entry, price)), best),
                Some((_, _bp)) => {
                    let new_second = match second {
                        Some((_, sp)) if better(price, sp) => Some((entry, price)),
                        None => Some((entry, price)),
                        _ => second,
                    };
                    (best, new_second)
                }
                None => (Some((entry, price)), second),
            }
        });

    let b = best?.0;
    let best_price = b.get("price")?.to_string();
    let best_size = b.get("size")?.to_string();

    let (second_price, second_size) = second
        .and_then(|(e, _)| {
            let p = e.get("price")?.to_string();
            let s = e.get("size")?.to_string();
            Some((p, s))
        })
        .unwrap_or_else(|| ("N/A".into(), "N/A".into()));

    Some(((best_price, best_size), (second_price, second_size)))
}

// ============================================================================
// Event Parsing
// ============================================================================

fn parse_event(message: String) -> Option<ParsedEvent> {
    let msg: WsMessage = serde_json::from_str(&message).ok()?;
    let result = msg.params?.result?;

    // just to double check!
    if result.topics.len() < 3 {
        return None;
    }

    let has_target = result
        .topics
        .get(2)
        .map(|t| t.eq_ignore_ascii_case(TARGET_TOPIC_HEX.as_str()))
        .unwrap_or(false);
    if !has_target {
        return None;
    }

    let hex_data = &result.data;
    if hex_data.len() < 2 + 64 * 4 {
        return None;
    }

    let (maker_id, maker_bytes) = parse_u256_hex_slice_with_bytes(hex_data, 2, 66)?;
    let (taker_id, taker_bytes) = parse_u256_hex_slice_with_bytes(hex_data, 66, 130)?;

    let (clob_id, token_bytes, maker_amt, taker_amt, base_type) =
        if maker_id.is_zero() && !taker_id.is_zero() {
            let m = parse_u256_hex_slice(hex_data, 130, 194)?;
            let t = parse_u256_hex_slice(hex_data, 194, 258)?;
            (taker_id, taker_bytes, m, t, "BUY")
        } else if taker_id.is_zero() && !maker_id.is_zero() {
            let m = parse_u256_hex_slice(hex_data, 130, 194)?;
            let t = parse_u256_hex_slice(hex_data, 194, 258)?;
            (maker_id, maker_bytes, m, t, "SELL")
        } else {
            return None;
        };

    let shares = if base_type == "BUY" {
        u256_to_f64(&taker_amt)?
    } else {
        u256_to_f64(&maker_amt)?
    } / 1e6;
    if shares <= 0.0 {
        return None;
    }

    let usd = if base_type == "BUY" {
        u256_to_f64(&maker_amt)?
    } else {
        u256_to_f64(&taker_amt)?
    } / 1e6;
    let price = usd / shares;

    let mut order_type = base_type.to_string();
    if result.topics[0].eq_ignore_ascii_case(ORDERS_FILLED_EVENT_SIGNATURE) {
        order_type.push_str("_FILL");
    }

    Some(ParsedEvent {
        block_number: result
            .block_number
            .as_deref()
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            .unwrap_or_default(),
        tx_hash: result.transaction_hash.unwrap_or_default(),
        order: OrderInfo {
            order_type,
            clob_token_id: u256_to_dec_cached(&token_bytes, &clob_id),
            usd_value: usd,
            shares,
            price_per_share: price,
        },
    })
}

// ============================================================================
// Hex Parsing Helpers
// ============================================================================

#[inline]
fn parse_u256_hex_slice_with_bytes(
    full: &str,
    start: usize,
    end: usize,
) -> Option<(U256, [u8; 32])> {
    let slice = full.get(start..end)?;
    let clean = slice.strip_prefix("0x").unwrap_or(slice);
    if clean.len() > 64 {
        return None;
    }

    let mut hex_buf = [b'0'; 64];
    hex_buf[64 - clean.len()..].copy_from_slice(clean.as_bytes());

    let mut out = [0u8; 32];
    for i in 0..32 {
        let hi = hex_nibble(hex_buf[i * 2])?;
        let lo = hex_nibble(hex_buf[i * 2 + 1])?;
        out[i] = (hi << 4) | lo;
    }
    Some((U256::from_be_slice(&out), out))
}

#[inline]
fn parse_u256_hex_slice(full: &str, start: usize, end: usize) -> Option<U256> {
    parse_u256_hex_slice_with_bytes(full, start, end).map(|(v, _)| v)
}

fn u256_to_dec_cached(bytes: &[u8; 32], val: &U256) -> Arc<str> {
    TOKEN_ID_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(s) = cache.get(bytes) {
            return Arc::clone(s);
        } // Cheap Arc clone
        let s: Arc<str> = val.to_string().into();
        cache.insert(*bytes, Arc::clone(&s));
        s
    })
}

fn u256_to_f64(v: &U256) -> Option<f64> {
    if v.bit_len() <= 64 {
        Some(v.as_limbs()[0] as f64)
    } else {
        v.to_string().parse().ok()
    }
}

// Hex nibble lookup table - 2-3x faster than branching
const HEX_NIBBLE_LUT: [u8; 256] = {
    let mut lut = [255u8; 256];
    let mut i = b'0';
    while i <= b'9' {
        lut[i as usize] = i - b'0';
        i += 1;
    }
    let mut i = b'a';
    while i <= b'f' {
        lut[i as usize] = i - b'a' + 10;
        i += 1;
    }
    let mut i = b'A';
    while i <= b'F' {
        lut[i as usize] = i - b'A' + 10;
        i += 1;
    }
    lut
};

#[inline(always)]
fn hex_nibble(b: u8) -> Option<u8> {
    let val = HEX_NIBBLE_LUT[b as usize];
    if val == 255 { None } else { Some(val) }
}

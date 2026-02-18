use anyhow::anyhow;
use polymarket_client_sdk::{
    clob::{Client, types::{OrderType, Side}},
    types::Decimal,
};
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;

use crate::{
    OrderResponse,
    config::site,
    models::{OrderInfo, ResubmitRequest, SizeType, WorkItem},
    orders,
    risk_guard::{RiskGuard, RiskGuardConfig, SafetyDecision, calc_liquidity_depth},
    settings::{
        FIXED_TRADE_VALUE, MIN_CASH_VALUE, MIN_SHARE_COUNT, MIN_WHALE_SHARES_TO_COPY,
        SCALING_RATIO, USE_PROBABILISTIC_SIZING, get_resubmit_max_buffer, get_tier_params,
        should_skip_trade,
    },
};

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
    client: Arc<Client>,
    private_key: Arc<String>,
    funder_address: Arc<String>,
    enable_trading: bool,
    mock_trading: bool,
    risk_config: RiskGuardConfig,
    resubmit_sender: mpsc::UnboundedSender<ResubmitRequest>,
) {
    std::thread::spawn(move || {
        let mut risk_guard = RiskGuard::new(risk_config);
        process_order_queue(
            work_receiver,
            client,
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
    client: Arc<Client>,
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
    client: &mut Client,
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
            let trade_side = if is_buy_order {
                Side::Buy
            } else {
                Side::Sell
            };
            match fetch_orderbook_liquidity_depth(
                client,
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
    client: &Client,
    token_id: &str,
    trade_side: Side,
    price_threshold: f64,
) -> Result<f64, &'static str> {
    let url = format!("{}/book?token_id={}", site::CLOB_API_BASE, token_id);
    let handle = tokio::runtime::Handle::current();
    let resp = handle
        .block_on(
            client
                .http_client()
                .get(&url)
                .timeout(Duration::from_millis(500))
                .send(),
        )
        .map_err(|_| "NETWORK")?;

    if !resp.status().is_success() {
        return Err("HTTP_ERROR");
    }

    let orderbook_data: Value = handle.block_on(resp.json()).map_err(|_| "PARSE")?;
    let orderbook_side_key = if trade_side == Side::Buy {
        "asks"
    } else {
        "bids"
    };

    // Stack array instead of Vec - avoids heap allocation for max 10 items
    let mut price_levels: [(f64, f64); 10] = [(0.0, 0.0); 10];
    let mut level_count = 0;
    if let Some(levels_array) = orderbook_data[orderbook_side_key].as_array() {
        for level in levels_array.iter().take(10) {
            if let (Some(price), Some(size)) = (
                level["price"].as_str().and_then(|s| s.parse().ok()),
                level["size"].as_str().and_then(|s| s.parse().ok()),
            ) {
                price_levels[level_count] = (price, size);
                level_count += 1;
            }
        }
    }

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

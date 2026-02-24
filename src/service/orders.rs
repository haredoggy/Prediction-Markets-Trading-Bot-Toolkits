use alloy::primitives::U256;
use alloy::signers::k256::ecdsa::SigningKey;
use alloy::signers::local::LocalSigner;
use anyhow::{Result, anyhow};
use chrono::DateTime;
use polymarket_client_sdk::auth::Normal;
use polymarket_client_sdk::auth::state::Authenticated;
use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::response::PostOrderResponse;
use polymarket_client_sdk::clob::types::{Amount, OrderType, Side};
use polymarket_client_sdk::types::Decimal;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

/// Place a buy order (market order) without API keys
/// Uses L1 authentication (signature only)
///
/// # Arguments
/// * `client` - The authenticated client
/// * `signer` - The signer for the client
/// * `token_id` - The token ID for the market outcome
/// * `usdc_amount` - Amount in USDC to spend
/// * `order_type` - Order type (FOK, GTC, etc.). Defaults to FOK if None
///
/// # Returns
/// The order response from Polymarket
pub async fn buy_order(
    client: &Client<Authenticated<Normal>>,
    signer: &mut LocalSigner<SigningKey>,
    token_id: &str,
    usdc_amount: Decimal,
    order_type: Option<OrderType>,
) -> Result<PostOrderResponse> {
    // Convert token_id string to U256
    let token_id_u256 = if token_id.starts_with("0x") {
        U256::from_str_radix(token_id.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Invalid token_id hex format: {}", e))?
    } else {
        U256::from_str(token_id).map_err(|e| anyhow!("Invalid token_id decimal format: {}", e))?
    };

    // Create market buy order using SDK builder
    let order_type_val = order_type.unwrap_or(OrderType::FOK);

    // Set expiration to at least 1 minute in the future (Polymarket requirement)
    let expiration_time = DateTime::from_timestamp(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 90, // 90 seconds in the future (1.5 minutes for safety)
        0,
    )
    .ok_or_else(|| anyhow!("Failed to create expiration timestamp"))?;

    let market_order = client
        .market_order()
        .token_id(token_id_u256)
        .amount(Amount::usdc(usdc_amount)?)
        .side(Side::Buy)
        .order_type(order_type_val)
        .expiration(expiration_time)
        .build()
        .await?;

    let signed = client.sign(signer, market_order).await?;

    // Use SDK's post_order which handles authentication automatically
    // The SDK's authenticate() method should create API keys if needed
    match client.post_order(signed).await {
        Ok(response) => {
            // Check if the response indicates insufficient balance/allowance
            if let Some(ref error_msg) = response.error_msg {
                if error_msg.contains("not enough balance") || error_msg.contains("allowance") {
                    return Err(anyhow!(
                        "Insufficient balance/allowance: {}. \
                        SOLUTION: Go to https://polymarket.com → Connect wallet → Make ANY test trade (even $1) → This will auto-approve USDC spending. \
                        OR manually approve USDC (0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174) for exchange contract (0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E) on Polygon. \
                        Required amount: {} USDC",
                        error_msg,
                        usdc_amount
                    ));
                }
            }
            Ok(response)
        }
        Err(e) => {
            // Check if error is due to insufficient balance/allowance
            let error_str = e.to_string();
            if error_str.contains("not enough balance")
                || error_str.contains("allowance")
                || error_str.contains("INSUFFICIENT")
                || error_str.contains("insufficient")
            {
                return Err(anyhow!(
                    "Insufficient balance/allowance. \
                    SOLUTION: Go to https://polymarket.com → Connect wallet → Make ANY test trade (even $1) → This will auto-approve USDC spending. \
                    OR manually approve USDC (0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174) for exchange contract (0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E) on Polygon. \
                    Required amount: {} USDC. Original error: {}",
                    usdc_amount,
                    error_str
                ));
            }
            // Re-throw other errors as anyhow error
            Err(anyhow::Error::from(e))
        }
    }
}

/// Place a sell order (limit order) without API keys
/// Uses L1 authentication (signature only)
///
/// # Arguments
/// * `client` - The authenticated client
/// * `signer` - The signer for the client
/// * `token_id` - The token ID for the market outcome
/// * `size` - Number of outcome tokens to sell
/// * `price` - Price per share in token terms (0.0 to 1.0)
/// * `order_type` - Order type (GTC, GTD, etc.). Defaults to GTC if None
///
/// # Returns
/// The order response from Polymarket
pub async fn sell_order(
    client: &Client<Authenticated<Normal>>,
    signer: &mut LocalSigner<SigningKey>,
    token_id: &str,
    size: Decimal,
    price: Decimal,
    order_type: Option<OrderType>,
) -> Result<PostOrderResponse> {
    // Convert token_id string to U256
    let token_id_u256 = if token_id.starts_with("0x") {
        U256::from_str_radix(token_id.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Invalid token_id hex format: {}", e))?
    } else {
        U256::from_str(token_id).map_err(|e| anyhow!("Invalid token_id decimal format: {}", e))?
    };

    let order_type_val = order_type.unwrap_or(OrderType::GTC);

    // Set expiration to at least 1 minute in the future (Polymarket requirement)
    let expiration_time = DateTime::from_timestamp(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 90, // 90 seconds in the future (1.5 minutes for safety)
        0,
    )
    .ok_or_else(|| anyhow!("Failed to create expiration timestamp"))?;

    let limit_order = client
        .limit_order()
        .token_id(token_id_u256)
        .size(size)
        .price(price)
        .side(Side::Sell)
        .order_type(order_type_val)
        .expiration(expiration_time)
        .build()
        .await?;

    let signed = client.sign(&signer, limit_order).await?;

    // Use SDK's post_order which handles authentication automatically
    match client.post_order(signed).await {
        Ok(response) => {
            // Check if the response indicates insufficient balance/allowance
            if let Some(ref error_msg) = response.error_msg {
                if error_msg.contains("not enough balance") || error_msg.contains("allowance") {
                    // For sell orders, this usually means Conditional Tokens aren't approved
                    return Err(anyhow!(
                        "Insufficient balance/allowance for SELL order: {}. \
                        SOLUTION: Your Gnosis Safe needs to approve Conditional Tokens for the exchange. \
                        Run: cargo run --release --bin approve_tokens \
                        OR manually approve through your Gnosis Safe: https://app.safe.global/ \
                        → Select your Safe → Apps → Transaction Builder \
                        → Approve Conditional Tokens (0x4d97dcd97ec945f40cf65f87097ace5ea0476045) \
                        for exchange (0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E) using setApprovalForAll",
                        error_msg
                    ));
                }
            }
            Ok(response)
        }
        Err(e) => {
            // Check if error is due to insufficient balance/allowance
            let error_str = e.to_string();
            if error_str.contains("not enough balance")
                || error_str.contains("allowance")
                || error_str.contains("INSUFFICIENT")
                || error_str.contains("insufficient")
            {
                return Err(anyhow!(
                    "Insufficient balance/allowance for SELL order. \
                    SOLUTION: Your Gnosis Safe needs to approve Conditional Tokens for the exchange. \
                    Run: cargo run --release --bin approve_tokens \
                    OR manually approve through your Gnosis Safe: https://app.safe.global/ \
                    → Select your Safe → Apps → Transaction Builder \
                    → Approve Conditional Tokens (0x4d97dcd97ec945f40cf65f87097ace5ea0476045) \
                    for exchange (0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E) using setApprovalForAll. \
                    Original error: {}",
                    error_str
                ));
            }
            // Re-throw other errors as anyhow error
            Err(anyhow::Error::from(e))
        }
    }
}

/// Place a buy limit order without API keys
/// Uses L1 authentication (signature only)
///
/// # Arguments
/// * `client` - The authenticated client
/// * `signer` - The signer for the client
/// * `token_id` - The token ID for the market outcome
/// * `size` - Number of outcome tokens to buy
/// * `price` - Maximum price per share in token terms (0.0 to 1.0)
/// * `order_type` - Order type (GTC, GTD, etc.). Defaults to GTC if None
///
/// # Returns
/// The order response from Polymarket
pub async fn buy_limit_order(
    client: &Client<Authenticated<Normal>>,
    signer: &mut LocalSigner<SigningKey>,
    token_id: &str,
    size: Decimal,
    price: Decimal,
    order_type: Option<OrderType>,
) -> Result<PostOrderResponse> {
    // Convert token_id string to U256
    let token_id_u256 = if token_id.starts_with("0x") {
        U256::from_str_radix(token_id.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Invalid token_id hex format: {}", e))?
    } else {
        U256::from_str(token_id).map_err(|e| anyhow!("Invalid token_id decimal format: {}", e))?
    };

    let order_type_val = order_type.unwrap_or(OrderType::GTC);

    // Set expiration to at least 1 minute in the future (Polymarket requirement)
    let expiration_time = DateTime::from_timestamp(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 90, // 90 seconds in the future (1.5 minutes for safety)
        0,
    )
    .ok_or_else(|| anyhow!("Failed to create expiration timestamp"))?;

    let limit_order = client
        .limit_order()
        .token_id(token_id_u256)
        .size(size)
        .price(price)
        .side(Side::Buy)
        .expiration(expiration_time)
        .order_type(order_type_val)
        .build()
        .await?;

    let signed = client.sign(&signer, limit_order).await?;

    // Use SDK's post_order which handles authentication automatically
    match client.post_order(signed).await {
        Ok(response) => {
            // Check if the response indicates insufficient balance/allowance
            if let Some(ref error_msg) = response.error_msg {
                if error_msg.contains("not enough balance") || error_msg.contains("allowance") {
                    // For sell orders, this usually means Conditional Tokens aren't approved
                    return Err(anyhow!(
                        "Insufficient balance/allowance for SELL order: {}. \
                        SOLUTION: Your Gnosis Safe needs to approve Conditional Tokens for the exchange. \
                        Run: cargo run --release --bin approve_tokens \
                        OR manually approve through your Gnosis Safe: https://app.safe.global/ \
                        → Select your Safe → Apps → Transaction Builder \
                        → Approve Conditional Tokens (0x4d97dcd97ec945f40cf65f87097ace5ea0476045) \
                        for exchange (0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E) using setApprovalForAll",
                        error_msg
                    ));
                }
            }
            Ok(response)
        }
        Err(e) => {
            // Check if error is due to insufficient balance/allowance
            let error_str = e.to_string();
            if error_str.contains("not enough balance")
                || error_str.contains("allowance")
                || error_str.contains("INSUFFICIENT")
                || error_str.contains("insufficient")
            {
                return Err(anyhow!(
                    "Insufficient balance/allowance for BUY LIMIT order. \
                    SOLUTION: Your Gnosis Safe needs to approve Conditional Tokens for the exchange. \
                    Run: cargo run --release --bin approve_tokens \
                    OR manually approve through your Gnosis Safe: https://app.safe.global/ \
                    → Select your Safe → Apps → Transaction Builder \
                    → Approve Conditional Tokens (0x4d97dcd97ec945f40cf65f87097ace5ea0476045) \
                    for exchange (0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E) using setApprovalForAll. \
                    Original error: {}",
                    error_str
                ));
            }
            // Re-throw other errors as anyhow error
            Err(anyhow::Error::from(e))
        }
    }
}

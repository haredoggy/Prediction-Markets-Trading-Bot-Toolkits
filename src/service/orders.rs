use anyhow::{Result, anyhow};
use tokio::sync::{mpsc, oneshot};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::DateTime;
use alloy::signers::{Signer as _, local::LocalSigner};
use alloy::primitives::{U256, Address};
use polymarket_client_sdk::POLYGON;
use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::{OrderType, Side, Amount, SignatureType};
use polymarket_client_sdk::clob::types::response::PostOrderResponse;
use polymarket_client_sdk::types::Decimal;

use crate::models::{ParsedEvent, ResubmitRequest, WorkItem};
use crate::config::settings::ORDER_REPLY_TIMEOUT;

// ============================================================================
// Order Engine
// ============================================================================

#[derive(Clone)]
pub struct OrderEngine {
    pub tx: mpsc::Sender<WorkItem>,
    #[allow(dead_code)]
    pub resubmit_tx: mpsc::UnboundedSender<ResubmitRequest>,
    pub enable_trading: bool,
}

impl OrderEngine {
    pub async fn submit(&self, evt: ParsedEvent, is_live: Option<bool>) -> String {
        if !self.enable_trading {
            return "SKIPPED_DISABLED".into();
        }

        let (resp_tx, resp_rx) = oneshot::channel();
        if let Err(e) = self.clone().tx.try_send(WorkItem {
            event: evt,
            respond_to: resp_tx,
            is_live,
        }) {
            return format!("QUEUE_ERR: {e}");
        }

        match tokio::time::timeout(ORDER_REPLY_TIMEOUT, resp_rx).await {
            Ok(Ok(msg)) => msg,
            Ok(Err(_)) => "WORKER_DROPPED".into(),
            Err(_) => "WORKER_TIMEOUT".into(),
        }
    }
}

/// Place a buy order (market order) without API keys
/// Uses L1 authentication (signature only)
/// 
/// # Arguments
/// * `private_key` - The private key for signing orders
/// * `funder_address` - The funder address that holds the USDC (where funds are)
/// * `token_id` - The token ID for the market outcome
/// * `usdc_amount` - Amount in USDC to spend
/// * `order_type` - Order type (FOK, GTC, etc.). Defaults to FOK if None
/// 
/// # Returns
/// The order response from Polymarket
pub async fn buy_order(
    private_key: &str,
    funder_address: &str,
    token_id: &str,
    usdc_amount: Decimal,
    order_type: Option<OrderType>,
) -> Result<PostOrderResponse> {

    let signer = LocalSigner::from_str(&private_key)?
        .with_chain_id(Some(POLYGON));

    // Parse funder address
    let funder_addr = Address::from_str(funder_address.trim_start_matches("0x"))
        .map_err(|e| anyhow!("Invalid funder_address format: {}", e))?;

    // Get signer address to compare with funder
    let signer_addr = signer.address();

    // Create unauthenticated client (for order building only)
    let client = Client::new("https://clob.polymarket.com", Default::default())?;

    // Convert token_id string to U256
    let token_id_u256 = if token_id.starts_with("0x") {
        U256::from_str_radix(token_id.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Invalid token_id hex format: {}", e))?
    } else {
        U256::from_str(token_id)
            .map_err(|e| anyhow!("Invalid token_id decimal format: {}", e))?
    };

    // Create market buy order using SDK builder
    let order_type_val = order_type.unwrap_or(OrderType::FOK);
    
    // We need to authenticate temporarily to build the order, but we'll post without API keys
    // Funder is a Gnosis Safe (proxy) address, signer is private key that can sign for the Safe
    let authenticated_client = if funder_addr == signer_addr {
        // Funder and signer are the same - SDK will use Eoa automatically
        client
            .authentication_builder(&signer)
            .authenticate()
            .await?
    } else {
        // Funder is a Gnosis Safe address (proxy wallet), signer is private key
        // Use GnosisSafe signature type for Gnosis Safe wallets
        client
            .authentication_builder(&signer)
            .funder(funder_addr)
            .signature_type(SignatureType::GnosisSafe)
            .authenticate()
            .await?
    };
    
    // Set expiration to at least 1 minute in the future (Polymarket requirement)
    let expiration_time = DateTime::from_timestamp(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64 + 90, // 90 seconds in the future (1.5 minutes for safety)
        0
    ).ok_or_else(|| anyhow!("Failed to create expiration timestamp"))?;
    
    let market_order = authenticated_client
        .market_order()
        .token_id(token_id_u256)
        .amount(Amount::usdc(usdc_amount)?)
        .side(Side::Buy)
        .order_type(order_type_val)
        .expiration(expiration_time)
        .build()
        .await?;

    let signed = authenticated_client.sign(&signer, market_order).await?;
    
    // Use SDK's post_order which handles authentication automatically
    // The SDK's authenticate() method should create API keys if needed
    match authenticated_client.post_order(signed).await {
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
            if error_str.contains("not enough balance") || 
               error_str.contains("allowance") ||
               error_str.contains("INSUFFICIENT") ||
               error_str.contains("insufficient") {
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
/// * `private_key` - The private key for signing orders
/// * `funder_address` - The funder address that holds the tokens (where funds are)
/// * `token_id` - The token ID for the market outcome
/// * `size` - Number of outcome tokens to sell
/// * `price` - Price per share in token terms (0.0 to 1.0)
/// * `order_type` - Order type (GTC, GTD, etc.). Defaults to GTC if None
/// 
/// # Returns
/// The order response from Polymarket
pub async fn sell_order(
    private_key: &str,
    funder_address: &str,
    token_id: &str,
    size: Decimal,
    price: Decimal,
    order_type: Option<OrderType>,
) -> Result<PostOrderResponse> {

    let signer = LocalSigner::from_str(&private_key)?
        .with_chain_id(Some(POLYGON));

    // Parse funder address
    let funder_addr = Address::from_str(funder_address.trim_start_matches("0x"))
        .map_err(|e| anyhow!("Invalid funder_address format: {}", e))?;

    // Get signer address to compare with funder
    let signer_addr = signer.address();

    // Create unauthenticated client (for order building only)
    let client = Client::new("https://clob.polymarket.com", Default::default())?;

    // Convert token_id string to U256
    let token_id_u256 = if token_id.starts_with("0x") {
        U256::from_str_radix(token_id.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Invalid token_id hex format: {}", e))?
    } else {
        U256::from_str(token_id)
            .map_err(|e| anyhow!("Invalid token_id decimal format: {}", e))?
    };

    // Authenticate temporarily to build and sign the order
    // Funder is a Gnosis Safe (proxy) address, signer is private key that can sign for the Safe
    let authenticated_client = if funder_addr == signer_addr {
        // Funder and signer are the same - SDK will use Eoa automatically
        client
            .authentication_builder(&signer)
            .authenticate()
            .await?
    } else {
        // Funder is a Gnosis Safe address (proxy wallet), signer is private key
        // Use GnosisSafe signature type for Gnosis Safe wallets
        client
            .authentication_builder(&signer)
            .funder(funder_addr)
            .signature_type(SignatureType::GnosisSafe)
            .authenticate()
            .await?
    };

    let order_type_val = order_type.unwrap_or(OrderType::GTC);
    
    // Set expiration to at least 1 minute in the future (Polymarket requirement)
    let expiration_time = DateTime::from_timestamp(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64 + 90, // 90 seconds in the future (1.5 minutes for safety)
        0
    ).ok_or_else(|| anyhow!("Failed to create expiration timestamp"))?;
    
    let limit_order = authenticated_client
        .limit_order()
        .token_id(token_id_u256)
        .size(size)
        .price(price)
        .side(Side::Sell)
        .order_type(order_type_val)
        .expiration(expiration_time)
        .build()
        .await?;

    let signed = authenticated_client.sign(&signer, limit_order).await?;
    
    // Use SDK's post_order which handles authentication automatically
    match authenticated_client.post_order(signed).await {
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
            if error_str.contains("not enough balance") || 
               error_str.contains("allowance") ||
               error_str.contains("INSUFFICIENT") ||
               error_str.contains("insufficient") {
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
/// * `private_key` - The private key for signing orders
/// * `funder_address` - The funder address that holds the USDC (where funds are)
/// * `token_id` - The token ID for the market outcome
/// * `size` - Number of outcome tokens to buy
/// * `price` - Maximum price per share in token terms (0.0 to 1.0)
/// * `order_type` - Order type (GTC, GTD, etc.). Defaults to GTC if None
/// 
/// # Returns
/// The order response from Polymarket
pub async fn buy_limit_order(
    private_key: &str,
    funder_address: &str,
    token_id: &str,
    size: Decimal,
    price: Decimal,
    order_type: Option<OrderType>,
) -> Result<PostOrderResponse> {

    let signer = LocalSigner::from_str(&private_key)?
        .with_chain_id(Some(POLYGON));

    // Parse funder address
    let funder_addr = Address::from_str(funder_address.trim_start_matches("0x"))
        .map_err(|e| anyhow!("Invalid funder_address format: {}", e))?;

    // Get signer address to compare with funder
    let signer_addr = signer.address();

    // Create unauthenticated client (for order building only)
    let client = Client::new("https://clob.polymarket.com", Default::default())?;

    // Convert token_id string to U256
    let token_id_u256 = if token_id.starts_with("0x") {
        U256::from_str_radix(token_id.trim_start_matches("0x"), 16)
            .map_err(|e| anyhow!("Invalid token_id hex format: {}", e))?
    } else {
        U256::from_str(token_id)
            .map_err(|e| anyhow!("Invalid token_id decimal format: {}", e))?
    };

    // Authenticate temporarily to build and sign the order
    // Funder is a Gnosis Safe (proxy) address, signer is private key that can sign for the Safe
    let authenticated_client = if funder_addr == signer_addr {
        // Funder and signer are the same - SDK will use Eoa automatically
        client
            .authentication_builder(&signer)
            .authenticate()
            .await?
    } else {
        // Funder is a Gnosis Safe address (proxy wallet), signer is private key
        // Use GnosisSafe signature type for Gnosis Safe wallets
        client
            .authentication_builder(&signer)
            .funder(funder_addr)
            .signature_type(SignatureType::GnosisSafe)
            .authenticate()
            .await?
    };

    let order_type_val = order_type.unwrap_or(OrderType::GTC);
    
    // Set expiration to at least 1 minute in the future (Polymarket requirement)
    let expiration_time = DateTime::from_timestamp(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64 + 90, // 90 seconds in the future (1.5 minutes for safety)
        0
    ).ok_or_else(|| anyhow!("Failed to create expiration timestamp"))?;
    
    let limit_order = authenticated_client
        .limit_order()
        .token_id(token_id_u256)
        .size(size)
        .price(price)
        .side(Side::Buy)
        .expiration(expiration_time)
        .order_type(order_type_val)
        .build()
        .await?;

    let signed = authenticated_client.sign(&signer, limit_order).await?;
    
    // Use SDK's post_order which handles authentication automatically
    match authenticated_client.post_order(signed).await {
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
            if error_str.contains("not enough balance") || 
               error_str.contains("allowance") ||
               error_str.contains("INSUFFICIENT") ||
               error_str.contains("insufficient") {
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
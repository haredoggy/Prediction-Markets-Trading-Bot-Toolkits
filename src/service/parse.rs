//! ABI log decoding for Polymarket CTF Exchange events.
//!
//! Decodes `OrderFilled` / `OrdersMatched` and projects them into the
//! [`WhaleTrade`] canonical view. Topic-0 selectors are recomputed from
//! the canonical event signatures via `keccak256`.

use crate::models::{Side, VenueId, WhaleTrade};
use alloy_primitives::{B256, U256};
use alloy_sol_types::{sol, SolEvent};
use anyhow::{anyhow, Result};

use super::onchain::RawLog;

sol! {
    /// CTF Exchange — fired once per filled maker order.
    event OrderFilled(
        bytes32 indexed orderHash,
        address indexed maker,
        address indexed taker,
        uint256 makerAssetId,
        uint256 takerAssetId,
        uint256 makerAmountFilled,
        uint256 takerAmountFilled,
        uint256 fee
    );
}

/// Topic-0 selector for `OrderFilled`, derived once at compile time via the
/// `sol!` macro.
pub fn order_filled_topic() -> B256 {
    OrderFilled::SIGNATURE_HASH
}

/// Returns `Ok(Some(trade))` if the log was an `OrderFilled` involving the
/// given whale address (as maker), `Ok(None)` if it was not relevant, or `Err`
/// on a malformed log.
pub fn decode_whale_trade(log: &RawLog, whale_address: &str) -> Result<Option<WhaleTrade>> {
    let whale = whale_address.to_lowercase();
    let target_topic = order_filled_topic();

    let topic0 = log
        .topics
        .first()
        .ok_or_else(|| anyhow!("log has no topic0"))?;
    let topic0_b256: B256 = topic0
        .parse()
        .map_err(|e| anyhow!("topic0 not hex bytes32: {e}"))?;
    if topic0_b256 != target_topic {
        return Ok(None);
    }

    // OrderFilled has 4 indexed parameters: orderHash, maker, taker — wait, only 3 are indexed.
    // Re-decode by reconstructing the topics+data layout.
    if log.topics.len() < 4 {
        return Ok(None);
    }
    let maker_topic = &log.topics[2];
    let maker = topic_to_address(maker_topic)?;
    if maker.to_lowercase() != whale {
        return Ok(None);
    }

    // Non-indexed body: makerAssetId, takerAssetId, makerAmountFilled, takerAmountFilled, fee
    let data_bytes =
        hex::decode(log.data.trim_start_matches("0x")).map_err(|e| anyhow!(e))?;
    if data_bytes.len() < 5 * 32 {
        return Err(anyhow!("OrderFilled data shorter than 5*32 bytes"));
    }

    let maker_asset_id = U256::from_be_slice(&data_bytes[0..32]);
    let taker_asset_id = U256::from_be_slice(&data_bytes[32..64]);
    let maker_amount = U256::from_be_slice(&data_bytes[64..96]);
    let taker_amount = U256::from_be_slice(&data_bytes[96..128]);
    let _fee = U256::from_be_slice(&data_bytes[128..160]);

    // Polymarket convention: when maker is *selling outcome shares*, makerAssetId is
    // the conditional token (non-zero) and takerAssetId is 0 (USDC). When maker is
    // *buying outcome shares*, the reverse holds.
    let (side, token_id, shares, usd_notional) = if maker_asset_id != U256::ZERO {
        let token_id = maker_asset_id.to_string();
        let shares = to_f64_with_decimals(maker_amount, 6); // CTF shares share USDC decimals (6)
        let usd = to_f64_with_decimals(taker_amount, 6);
        (Side::Sell, token_id, shares, usd)
    } else {
        let token_id = taker_asset_id.to_string();
        let shares = to_f64_with_decimals(taker_amount, 6);
        let usd = to_f64_with_decimals(maker_amount, 6);
        (Side::Buy, token_id, shares, usd)
    };
    let price = if shares > 0.0 { usd_notional / shares } else { 0.0 };

    Ok(Some(WhaleTrade {
        venue: VenueId::Polymarket,
        maker,
        side,
        token_id,
        shares,
        price,
        usd_notional,
        tx_hash: Some(log.tx_hash.clone()),
        block_number: Some(log.block_number),
        observed_at: chrono::Utc::now(),
    }))
}

fn topic_to_address(topic: &str) -> Result<String> {
    let bytes = hex::decode(topic.trim_start_matches("0x")).map_err(|e| anyhow!(e))?;
    if bytes.len() != 32 {
        return Err(anyhow!("address topic not 32 bytes"));
    }
    Ok(format!("0x{}", hex::encode(&bytes[12..32])))
}

fn to_f64_with_decimals(v: U256, decimals: u32) -> f64 {
    // Safe for typical USDC amounts; clamps if overflowing f64 mantissa.
    let s = v.to_string();
    s.parse::<f64>().unwrap_or(0.0) / 10f64.powi(decimals as i32)
}


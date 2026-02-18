use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use hmac::{Hmac, Mac};
use itoa::Buffer as ItoaBuffer;
use polymarket_client_sdk::auth::{Credentials, ExposeSecret};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

pub mod utils;
pub mod config;
pub mod service;
pub mod models;
pub mod bot;
pub mod ui;

type HmacSha256 = Hmac<Sha256>;

use std::cell::RefCell;

thread_local! {
    static MESSAGE_BUF: RefCell<String> = RefCell::new(String::with_capacity(256));
    static ITOA_BUF: RefCell<ItoaBuffer> = RefCell::new(ItoaBuffer::new());
    static SIGNATURE_BUF: RefCell<String> = RefCell::new(String::with_capacity(48));
    static URL_BUF: RefCell<String> = RefCell::new(String::with_capacity(128));
    static JSON_BUF: RefCell<String> = RefCell::new(String::with_capacity(1024));
}

// ============================================================================
// ORDER RESPONSE (for parsing FAK order results)
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    pub success: bool,
    #[serde(rename = "errorMsg", default)]
    pub error_msg: String,
    #[serde(rename = "orderID", default)]
    pub order_id: String,
    #[serde(rename = "transactionsHashes", default)]
    pub transactions_hashes: Vec<String>,
    #[serde(default)]
    pub status: String,
    #[serde(rename = "takingAmount", default)]
    pub taking_amount: String,
    #[serde(rename = "makingAmount", default)]
    pub making_amount: String,
}

// ============================================================================
// PREPARED CREDENTIALS
// ============================================================================

#[derive(Clone)]
pub struct PreparedCreds {
    pub api_key: String,
    pub api_passphrase: String,
    hmac_template: HmacSha256,
}

impl PreparedCreds {
    pub fn from_api_creds(creds: &Credentials) -> Result<Self> {
        let decoded_secret: Vec<u8> = URL_SAFE.decode(creds.secret().expose_secret().as_bytes())?;
        let hmac_template = HmacSha256::new_from_slice(&decoded_secret)
            .map_err(|e| anyhow!("Invalid HMAC key: {}", e))?;
        Ok(Self {
            api_key: creds.key().to_string(),
            api_passphrase: creds.passphrase().expose_secret().to_string(),
            hmac_template,
        })
    }

    #[inline]
    pub fn sign_raw(&self, message: &[u8]) -> [u8; 32] {
        let mut mac = self.hmac_template.clone();
        mac.update(message);
        let result = mac.finalize();
        let bytes = result.into_bytes();
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        out
    }

    #[inline]
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.sign_raw(message).to_vec()
    }

    #[inline]
    pub fn sign_b64_fast(&self, message: &[u8]) -> String {
        let raw = self.sign_raw(message);
        URL_SAFE.encode(raw)
    }

    #[inline]
    pub fn sign_b64(&self, message: &[u8]) -> String {
        self.sign_b64_fast(message)
    }
}

#[derive(Debug, Clone)]
pub struct OrderArgs {
    pub token_id: String,
    pub price: f64,
    pub size: f64,
    pub side: String,
    pub fee_rate_bps: Option<i64>,
    pub nonce: Option<i64>,
    pub expiration: Option<String>,
    pub taker: Option<String>,
    pub order_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderStruct {
    salt: u128,
    maker: String,
    signer: String,
    taker: String,
    #[serde(rename = "tokenId")]
    token_id: String,
    #[serde(rename = "makerAmount")]
    maker_amount: String,
    #[serde(rename = "takerAmount")]
    taker_amount: String,
    expiration: String,
    nonce: String,
    #[serde(rename = "feeRateBps")]
    fee_rate_bps: String,
    side: i32,
    #[serde(rename = "signatureType")]
    signature_type: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignedOrder {
    pub order: OrderStruct,
    pub signature: String,
}

impl SignedOrder {
    pub fn post_body(&self, owner: &str, order_type: &str) -> String {
        self.post_body_with_owner(Some(owner), order_type)
    }

    /// Post body without owner (for L1 auth, no API keys)
    pub fn post_body_no_owner(&self, order_type: &str) -> String {
        self.post_body_with_owner(None, order_type)
    }

    fn post_body_with_owner(&self, owner: Option<&str>, order_type: &str) -> String {
        JSON_BUF.with(|json_buf| {
            ITOA_BUF.with(|itoa_buf| {
                let mut buf = json_buf.borrow_mut();
                let mut itoa = itoa_buf.borrow_mut();

                buf.clear();
                buf.reserve(512);

                let side_str = if self.order.side == 0 { "BUY" } else { "SELL" };

                buf.push_str(r#"{"order":{"salt":"#);
                buf.push_str(itoa.format(self.order.salt));
                buf.push_str(r#","maker":""#);
                buf.push_str(&self.order.maker);
                buf.push_str(r#"","signer":""#);
                buf.push_str(&self.order.signer);
                buf.push_str(r#"","taker":""#);
                buf.push_str(&self.order.taker);
                buf.push_str(r#"","tokenId":""#);
                buf.push_str(&self.order.token_id);
                buf.push_str(r#"","makerAmount":""#);
                buf.push_str(&self.order.maker_amount);
                buf.push_str(r#"","takerAmount":""#);
                buf.push_str(&self.order.taker_amount);
                buf.push_str(r#"","expiration":""#);
                buf.push_str(&self.order.expiration);
                buf.push_str(r#"","nonce":""#);
                buf.push_str(&self.order.nonce);
                buf.push_str(r#"","feeRateBps":""#);
                buf.push_str(&self.order.fee_rate_bps);
                buf.push_str(r#"","side":""#);
                buf.push_str(side_str);
                buf.push_str(r#"","signatureType":"#);
                buf.push_str(itoa.format(self.order.signature_type));
                buf.push_str(r#","signature":""#);
                buf.push_str(&self.signature);
                buf.push_str(r#""}"#);
                if let Some(owner_val) = owner {
                    buf.push_str(r#","owner":""#);
                    buf.push_str(owner_val);
                }
                buf.push_str(r#","orderType":""#);
                buf.push_str(order_type);
                buf.push_str(r#""}"#);

                buf.clone()
            })
        })
    }
}

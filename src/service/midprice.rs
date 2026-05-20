//! Midprice quote source for the position monitor.
//!
//! Polymarket exposes a `/midpoint?token_id=X` endpoint that returns the
//! mid between best bid and best ask for a single outcome token. Cheap and
//! exactly what TP/SL polling needs.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

#[async_trait]
pub trait MidpriceSource: Send + Sync {
    async fn midprice(&self, token_id: &str) -> Result<f64>;
}

pub struct ClobMidpriceSource {
    http: Client,
    clob_base: String,
}

impl ClobMidpriceSource {
    pub fn new(http: Client, clob_base: impl Into<String>) -> Self {
        Self {
            http,
            clob_base: clob_base.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct MidpointResponse {
    mid: String,
}

#[async_trait]
impl MidpriceSource for ClobMidpriceSource {
    async fn midprice(&self, token_id: &str) -> Result<f64> {
        let url = format!("{}/midpoint?token_id={}", self.clob_base, token_id);
        let resp = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<MidpointResponse>()
            .await?;
        resp.mid
            .parse::<f64>()
            .map_err(|e| anyhow!("midpoint not a float: {e}"))
    }
}

//! In-memory cache for Polymarket market metadata (slug → CLOB token IDs).
//!
//! Backed by the Gamma API. The cache is read-through with a per-entry TTL;
//! callers never await against the network unless the key is missing or stale.

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

const DEFAULT_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone)]
pub struct MarketInfo {
    pub slug: String,
    pub question: String,
    pub yes_token_id: String,
    pub no_token_id: String,
    pub closed: bool,
}

#[derive(Debug)]
pub struct MarketCache {
    http: Client,
    gamma_base: String,
    entries: RwLock<HashMap<String, (MarketInfo, Instant)>>,
    ttl: Duration,
}

impl MarketCache {
    pub fn new(http: Client, gamma_base: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            http,
            gamma_base: gamma_base.into(),
            entries: RwLock::new(HashMap::new()),
            ttl: DEFAULT_TTL,
        })
    }

    pub async fn by_slug(&self, slug: &str) -> Result<MarketInfo> {
        if let Some(info) = self.peek(slug) {
            return Ok(info);
        }
        let info = self.fetch_by_slug(slug).await?;
        self.insert(slug.to_string(), info.clone());
        Ok(info)
    }

    fn peek(&self, slug: &str) -> Option<MarketInfo> {
        let g = self.entries.read();
        g.get(slug).and_then(|(info, when)| {
            if when.elapsed() < self.ttl {
                Some(info.clone())
            } else {
                None
            }
        })
    }

    fn insert(&self, slug: String, info: MarketInfo) {
        let mut g = self.entries.write();
        g.insert(slug, (info, Instant::now()));
    }

    async fn fetch_by_slug(&self, slug: &str) -> Result<MarketInfo> {
        let url = format!("{}/markets?slug={}", self.gamma_base, slug);
        let resp = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        let markets = resp
            .as_array()
            .ok_or_else(|| anyhow!("gamma /markets returned non-array"))?;
        let m = markets
            .iter()
            .find(|m| m.get("slug").and_then(|v| v.as_str()) == Some(slug))
            .ok_or_else(|| anyhow!("slug {slug} not found in gamma response"))?;

        parse_market(m)
    }
}

#[derive(Debug, Deserialize)]
struct ClobTokenIds(Vec<String>);

fn parse_market(m: &serde_json::Value) -> Result<MarketInfo> {
    let slug = m
        .get("slug")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("market missing slug"))?
        .to_string();
    let question = m
        .get("question")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let closed = m.get("closed").and_then(|v| v.as_bool()).unwrap_or(false);

    // Gamma stores the two CLOB token IDs as a JSON-encoded string array.
    // Decode defensively — older endpoints return them as a real array.
    let raw_tokens = m
        .get("clobTokenIds")
        .ok_or_else(|| anyhow!("market missing clobTokenIds"))?;
    let tokens: Vec<String> = match raw_tokens {
        serde_json::Value::String(s) => serde_json::from_str::<ClobTokenIds>(s)?.0,
        serde_json::Value::Array(_) => serde_json::from_value(raw_tokens.clone())?,
        _ => return Err(anyhow!("unrecognised clobTokenIds shape")),
    };
    if tokens.len() < 2 {
        return Err(anyhow!("expected 2 clob token ids, got {}", tokens.len()));
    }

    Ok(MarketInfo {
        slug,
        question,
        yes_token_id: tokens[0].clone(),
        no_token_id: tokens[1].clone(),
        closed,
    })
}

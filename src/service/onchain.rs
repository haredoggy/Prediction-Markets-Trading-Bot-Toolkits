//! Polygon WebSocket subscription.
//!
//! Subscribes to `eth_subscribe`/`logs` on the configured Polygon WS endpoint
//! and forwards each matching log as a [`RawLog`] over a tokio channel. Filter
//! parameters are pushed server-side so the bot is woken only when the watched
//! whale interacts with the configured CTF exchange contracts.

use anyhow::{anyhow, Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct RawLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub tx_hash: String,
    pub block_number: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogFilter {
    pub address: Vec<String>,
    pub topics: Vec<Option<Vec<String>>>,
}

/// Spawns a task that maintains a Polygon WS connection and forwards matching
/// logs into `tx`. Returns the JoinHandle so the caller can await shutdown.
pub fn spawn_subscription(
    ws_url: String,
    filter: LogFilter,
    tx: mpsc::Sender<RawLog>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut backoff_secs = 1u64;
        loop {
            match run_once(&ws_url, &filter, &tx).await {
                Ok(()) => {
                    // Server closed cleanly — reconnect after short pause.
                    warn!("polygon WS subscription ended; reconnecting");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    backoff_secs = 1;
                }
                Err(e) => {
                    error!(error = ?e, backoff_secs, "polygon WS error; reconnecting");
                    tokio::time::sleep(std::time::Duration::from_secs(backoff_secs))
                        .await;
                    backoff_secs = (backoff_secs * 2).min(30);
                }
            }
        }
    })
}

async fn run_once(
    ws_url: &str,
    filter: &LogFilter,
    tx: &mpsc::Sender<RawLog>,
) -> Result<()> {
    let (mut ws, _resp) = tokio_tungstenite::connect_async(ws_url)
        .await
        .context("connecting polygon ws")?;
    info!(url = %ws_url, "polygon WS connected");

    let sub_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_subscribe",
        "params": ["logs", filter]
    });
    ws.send(Message::Text(sub_req.to_string())).await?;

    while let Some(msg) = ws.next().await {
        let msg = msg?;
        let text = match msg {
            Message::Text(t) => t,
            Message::Binary(b) => String::from_utf8_lossy(&b).into_owned(),
            Message::Ping(p) => {
                ws.send(Message::Pong(p)).await?;
                continue;
            }
            Message::Close(_) => return Ok(()),
            _ => continue,
        };

        let value: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                debug!(?e, "non-JSON frame from WS");
                continue;
            }
        };

        // Subscription acknowledgement or response — ignore.
        if value.get("method").and_then(|v| v.as_str()) != Some("eth_subscription")
        {
            continue;
        }

        let params = value
            .get("params")
            .and_then(|p| p.get("result"))
            .ok_or_else(|| anyhow!("missing params.result"))?;

        let log = parse_log(params)?;
        if tx.send(log).await.is_err() {
            // Consumer dropped — exit cleanly.
            return Ok(());
        }
    }

    Ok(())
}

fn parse_log(v: &Value) -> Result<RawLog> {
    #[derive(Deserialize)]
    struct LogShape {
        address: String,
        topics: Vec<String>,
        data: String,
        #[serde(rename = "transactionHash")]
        tx_hash: String,
        #[serde(rename = "blockNumber")]
        block_number: String,
    }
    let shape: LogShape = serde_json::from_value(v.clone())?;
    let block_number = u64::from_str_radix(shape.block_number.trim_start_matches("0x"), 16)
        .unwrap_or(0);
    Ok(RawLog {
        address: shape.address.to_lowercase(),
        topics: shape.topics.into_iter().map(|t| t.to_lowercase()).collect(),
        data: shape.data,
        tx_hash: shape.tx_hash,
        block_number,
    })
}

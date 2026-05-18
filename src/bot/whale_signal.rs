//! On-chain whale signal bot — pure on-chain signal, ahead of public APIs.
//!
//! 🚧 In development. The on-chain WS subscription, log parsing, and
//! execution engine are production-ready (the [`copy_trading`][crate::bot::copy_trading]
//! bot uses them). What's missing here is the *fan-out* mode: subscribing to
//! many whales simultaneously and routing each fan-out trade through its own
//! risk bucket.

use crate::config::AppConfig;
use anyhow::Result;
use tracing::info;

pub async fn run(_cfg: AppConfig) -> Result<()> {
    info!("🚧 Whale signal bot — in development. See README #10.");
    Ok(())
}

//! BTC 5-min / 15-min / 1-hr arbitrage bot.
//!
//! 🚧 In development. The execution engine, signing, and risk layer are
//! production-ready (see [`copy_trading`][crate::bot::copy_trading]); the
//! signal generation for this strategy is the next piece on the roadmap.

use crate::config::AppConfig;
use anyhow::Result;
use tracing::info;

pub async fn run(_cfg: AppConfig) -> Result<()> {
    info!("🚧 BTC Up/Down arbitrage bot — in development. See README #2.");
    Ok(())
}

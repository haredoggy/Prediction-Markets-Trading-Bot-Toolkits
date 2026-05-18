//! Polymarket ↔ Kalshi cross-venue arbitrage bot.
//!
//! 🚧 In development. Hedged-leg execution shares the same signing/risk core
//! as [`copy_trading`][crate::bot::copy_trading]; the Kalshi adapter is the
//! next piece.

use crate::config::AppConfig;
use anyhow::Result;
use tracing::info;

pub async fn run(_cfg: AppConfig) -> Result<()> {
    info!("🚧 Cross-venue arbitrage bot — in development. See README #3.");
    Ok(())
}

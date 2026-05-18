//! Spread-farming bot — systematic bid-ask capture.
//!
//! 🚧 In development. Quoting engine + per-trade P&L tracking next.

use crate::config::AppConfig;
use anyhow::Result;
use tracing::info;

pub async fn run(_cfg: AppConfig) -> Result<()> {
    info!("🚧 Spread farming bot — in development. See README #5.");
    Ok(())
}

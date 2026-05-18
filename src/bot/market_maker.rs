//! Market making bot — two-sided GTD quoting with inventory skew.
//!
//! 🚧 In development. Quoting loop + inventory rebalance + per-fill P&L next.

use crate::config::AppConfig;
use anyhow::Result;
use tracing::info;

pub async fn run(_cfg: AppConfig) -> Result<()> {
    info!("🚧 Market making bot — in development. See README #9.");
    Ok(())
}

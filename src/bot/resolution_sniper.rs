//! Resolution-sniper bot — near-certainty buys held to $1.00 payout.
//!
//! 🚧 In development. Market scanner + certainty/depth filter on top of the
//! shared executor.

use crate::config::AppConfig;
use anyhow::Result;
use tracing::info;

pub async fn run(_cfg: AppConfig) -> Result<()> {
    info!("🚧 Resolution sniper bot — in development. See README #7.");
    Ok(())
}

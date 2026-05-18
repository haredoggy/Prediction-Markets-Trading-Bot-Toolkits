//! Orderbook imbalance bot — fade the dominant side.
//!
//! 🚧 In development. Live orderbook OBI tracker + thresholded entries.

use crate::config::AppConfig;
use anyhow::Result;
use tracing::info;

pub async fn run(_cfg: AppConfig) -> Result<()> {
    info!("🚧 Orderbook imbalance bot — in development. See README #8.");
    Ok(())
}

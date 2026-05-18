//! Direction-hunting bot — short-window momentum / flow setups.
//!
//! 🚧 In development. Scanner + TP/SL exit manager are the next pieces.

use crate::config::AppConfig;
use anyhow::Result;
use tracing::info;

pub async fn run(_cfg: AppConfig) -> Result<()> {
    info!("🚧 Direction hunting bot — in development. See README #4.");
    Ok(())
}

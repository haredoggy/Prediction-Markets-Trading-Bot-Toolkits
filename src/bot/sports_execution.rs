//! Sports betting execution bot — click-to-FAK interface.
//!
//! 🚧 In development. Live odds feed + manual click UI on top of the
//! production order executor.

use crate::config::AppConfig;
use anyhow::Result;
use tracing::info;

pub async fn run(_cfg: AppConfig) -> Result<()> {
    info!("🚧 Sports betting bot — in development. See README #6.");
    Ok(())
}

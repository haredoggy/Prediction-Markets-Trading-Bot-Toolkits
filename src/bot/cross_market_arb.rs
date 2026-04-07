// ============================================================================
// Cross-Market Arbitrage Bot  (Polymarket ↔ Kalshi)
// ============================================================================
//
// Watches the same market on both Polymarket and Kalshi simultaneously.
// When a configurable price delta (edge) is detected, it executes hedged
// legs on both venues — buying the cheaper side, selling the expensive side.
//
// Status: 🚧 In development

use anyhow::Result;
use tokio::sync::mpsc;

use crate::{
    config::settings::AppConfig,
    ui::components::logs::{LogEntry, LogLevel},
};

pub async fn run_bot(app_config: AppConfig, log_tx: mpsc::UnboundedSender<LogEntry>) -> Result<()> {
    let _ = log_tx.send(LogEntry::new(
        "--- Cross-Market Arbitrage Bot (Polymarket ↔ Kalshi) ---".to_string(),
        LogLevel::Info,
    ));
    let _ = log_tx.send(LogEntry::new(
        "Watches matching markets on both venues for price divergence.".to_string(),
        LogLevel::Info,
    ));
    let _ = log_tx.send(LogEntry::new(
        "Executes hedged legs when edge >= configured threshold.".to_string(),
        LogLevel::Info,
    ));
    let _ = log_tx.send(LogEntry::new(
        "Status: coming in the next release. Stay tuned.".to_string(),
        LogLevel::Warning,
    ));
    Ok(())
}

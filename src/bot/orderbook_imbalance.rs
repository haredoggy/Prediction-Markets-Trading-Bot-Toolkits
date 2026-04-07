// ============================================================================
// Orderbook Imbalance Bot
// ============================================================================
//
// Monitors the live bid/ask depth ratio (Order Book Imbalance = OBI) across
// configured markets. When OBI exceeds a configurable threshold in either
// direction, the bot fades into the imbalance — buying into heavy bids,
// selling into heavy asks.
//
// No external data feeds required: signal is derived purely from the live
// orderbook, making this strategy self-contained and low-latency.
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
        "--- Orderbook Imbalance Bot ---".to_string(),
        LogLevel::Info,
    ));
    let _ = log_tx.send(LogEntry::new(
        "Detects bid/ask depth imbalances and fades in the dominant direction.".to_string(),
        LogLevel::Info,
    ));
    let _ = log_tx.send(LogEntry::new(
        "Signal derived from live orderbook only — no external feeds needed.".to_string(),
        LogLevel::Info,
    ));
    let _ = log_tx.send(LogEntry::new(
        "Configurable: OBI threshold, book refresh rate, max position size.".to_string(),
        LogLevel::Info,
    ));
    let _ = log_tx.send(LogEntry::new(
        "Status: coming in the next release. Stay tuned.".to_string(),
        LogLevel::Warning,
    ));
    Ok(())
}

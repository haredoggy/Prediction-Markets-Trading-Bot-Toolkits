use anyhow::Result;
use tokio::sync::mpsc;

use polymarket_toolkits::bot;
use polymarket_toolkits::config::settings::AppConfig;
use polymarket_toolkits::ui::components::logs::{LogEntry, LogLevel};
use polymarket_toolkits::ui::layout::{BotType, run_bot_ui, run_selection_ui};

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize Rustls crypto provider (required for Rustls 0.23+)
    // This must be called before any TLS operations
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Show bot selection UI
    let selected_bot = run_selection_ui()?;

    match selected_bot {
        Some(BotType::CopyTrading) => {
            let app_config = AppConfig::load()?;

            println!("Initializing Copy Trading Bot...");
            // Create log channel
            let (log_tx, log_rx) = mpsc::unbounded_channel();

            // Send initial log
            let _ = log_tx.send(LogEntry::new(
                "Initializing Copy Trading Bot...".to_string(),
                LogLevel::Info,
            ));

            // Create bot task
            let log_tx_clone = log_tx.clone();
            let bot_task =
                async move { bot::copy_trading::run_bot(app_config, log_tx_clone).await };

            // Run UI with bot
            run_bot_ui("Copy Trading Bot".to_string(), log_rx, bot_task).await?;
        }
        Some(BotType::Arbitrage) => {
            let app_config = AppConfig::load()?;
            bot::arbitrage::run_bot(app_config).await?;
        }
        Some(BotType::Sniper) => {
            let app_config = AppConfig::load()?;
            bot::sniper::run_bot(app_config).await?;
        }
        None => {
            println!("No bot selected. Exiting.");
        }
    }

    Ok(())
}

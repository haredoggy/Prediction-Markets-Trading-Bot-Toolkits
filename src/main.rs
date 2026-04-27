use anyhow::Result;
use tokio::sync::mpsc;

use polymarket_toolkits::bot;
use polymarket_toolkits::config::settings::AppConfig;
use polymarket_toolkits::service::positions::init_rate_limiter;
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
            init_rate_limiter(app_config.trading.rate_limit, 10);

            println!("Initializing Copy Trading Bot...");
            let (log_tx, log_rx) = mpsc::unbounded_channel();
            let _ = log_tx.send(LogEntry::new(
                "Initializing Copy Trading Bot...".to_string(),
                LogLevel::Info,
            ));

            let bot_task =
                async move { bot::copy_trading::run_bot(app_config, log_tx).await };

            run_bot_ui("Copy Trading Bot".to_string(), log_rx, bot_task).await?;
        }
        Some(BotType::Arbitrage) => {
            let app_config = AppConfig::load()?;
            init_rate_limiter(app_config.trading.rate_limit, 10);

            let (log_tx, log_rx) = mpsc::unbounded_channel();
            let _ = log_tx.send(LogEntry::new(
                "Initializing Arbitrage Bot...".to_string(),
                LogLevel::Info,
            ));

            let bot_task =
                async move { bot::arbitrage::run_bot(app_config, log_tx).await };
            run_bot_ui("Arbitrage Bot".to_string(), log_rx, bot_task).await?;
        }
        Some(BotType::MarketMaker) => {
            let app_config = AppConfig::load()?;
            init_rate_limiter(app_config.trading.rate_limit, 10);

            let (log_tx, log_rx) = mpsc::unbounded_channel();
            let _ = log_tx.send(LogEntry::new(
                "Initializing Market Maker Bot...".to_string(),
                LogLevel::Info,
            ));

            let bot_task =
                async move { bot::market_maker::run_bot(app_config, log_tx).await };
            run_bot_ui("Market Maker Bot".to_string(), log_rx, bot_task).await?;
        }
        None => {
            println!("No bot selected. Exiting.");
        }
    }

    Ok(())
}

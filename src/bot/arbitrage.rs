// ============================================================================
// Arbitrage Bot
// ============================================================================

use std::{thread::sleep, time::Duration};

use anyhow::Result;
use hmac::digest::const_oid::Arc;
use tokio::sync::{Mutex, mpsc};

use crate::{
    config::{coin::{display_coin_selection, get_available_coins}, settings::AppConfig},
    ui::components::logs::{LogEntry, LogLevel}, utils::keyboard::{KeyAction, KeyboardHandler},
};

pub async fn run_bot(app_config: AppConfig, log_tx: mpsc::UnboundedSender<LogEntry>) -> Result<()> {
    let _ = log_tx.send(LogEntry::new(
        "--- Polymarket Arbitrage Bot - 15-Minute Market Monitor ---".to_string(),
        LogLevel::Info,
    ));

    // Step 1: User picks a coin via interactive menu (FYI: arrow keys + Enter)
    let coins = get_available_coins();
    let mut selected_index = 0;
    let mut keyboard = KeyboardHandler::new();
    keyboard.enable()?; // Enable raw mode for arrow key detection

    let selected_coin = loop {
        display_coin_selection(selected_index, log_tx.clone()); // Render menu with current selection

        match keyboard.read_key()? {
            KeyAction::Up => {
                // Wrap around to bottom if at top (FYI: circular navigation)
                selected_index = (selected_index + coins.len() - 1) % coins.len();
            }
            KeyAction::Down => {
                // Wrap around to top if at bottom
                selected_index = (selected_index + 1) % coins.len();
            }
            KeyAction::Enter => {
                keyboard.disable()?; // Clean up before returning
                break coins[selected_index].to_string();
            }
            KeyAction::Exit => {
                keyboard.disable()?;
                std::process::exit(0); // Ctrl+C exit
            }
            _ => {} // Ignore other keys
        }
    };

    let _ = log_tx.send(LogEntry::new(  
         format!(
            "\n✓ Coin selected: {}\n  Bot will automatically switch to next market when current market closes.\n  Press Ctrl+C to stop.\n\n",
            selected_coin
        )
        .to_string(),
        LogLevel::Info,
    ));

    // Step 2: Start continuous monitoring loop
    let mut ws: Option<Arc<MarketWebSocket>> = None; // WS connection (lazy init)
    let clob_client = Arc::new(Mutex::new(None::<Arc<services::create_clob_client::ClobClient>>)); // Trading client (lazy init)
    let monitor = Arc::new(Mutex::new(PriceMonitor::new())); // Price history tracker
    let recent_opportunities = Arc::new(Mutex::new(HashSet::new())); // Dedup tracker (prevents duplicate trades)
    let is_executing_trade = Arc::new(Mutex::new(false)); // Trade lock (prevents concurrent executions)

    loop {
        match discover_and_monitor(coin, &mut ws, &clob_client, &monitor, &recent_opportunities, &is_executing_trade, env).await {
            Ok(Some(market)) => {
                // Monitor until market closes (BTW: auto-finds next market after)
                while let Some(ref m) = market {
                    let end_date = chrono::DateTime::parse_from_rfc3339(&m.end_date)
                        .unwrap_or_else(|_| chrono::Utc::now().into())
                        .with_timezone(&chrono::Utc);
                    let now = chrono::Utc::now();
                    let time_until_end = (end_date - now).num_milliseconds();

                    if time_until_end <= 0 {
                        let _ = log_tx.send(LogEntry::new(
                            format!("Market: {}\nCoin: {}\nEnd Time: {}\nStatus: Searching for next market...\n", m.slug, coin, end_date.format("%Y-%m-%d %H:%M:%S UTC")).to_string(),
                            LogLevel::Info,
                        ));
                        break;
                    }

                    sleep(Duration::from_secs(1)).await;
                }
            }
            Ok(None) => {
                let _ = log_tx.send(LogEntry::new("Waiting 10 seconds before retrying...\n".to_string(), LogLevel::Info));
                sleep(Duration::from_secs(10)).await;
            }
            Err(e) => {
                let _ = log_tx.send(LogEntry::new(format!("Error: {}", e).to_string(), LogLevel::Error));
                sleep(Duration::from_secs(10)).await;
            }
        }
    }

    Ok(())
}

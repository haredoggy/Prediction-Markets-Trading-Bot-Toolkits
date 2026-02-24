// ============================================================================
// Copy Trading Bot
// ============================================================================

use anyhow::Result;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{Semaphore, mpsc};

use crate::{
    config::settings::AppConfig,
    service::{
        positions::{detect_position_changes, fetch_positions_for_wallet},
        trader::CopyTrader,
    },
    ui::components::logs::{LogEntry, LogLevel},
};

pub async fn run_bot(app_config: AppConfig, log_tx: mpsc::UnboundedSender<LogEntry>) -> Result<()> {
    let _ = log_tx.send(LogEntry::new(
        "Creating authenticated client...".to_string(),
        LogLevel::Info,
    ));

    // let risk_config = app_config.risk_guard_config();

    let _ = log_tx.send(LogEntry::new(
        "Initializing order processing channels...".to_string(),
        LogLevel::Info,
    ));

    let copy_trader = CopyTrader::new(&app_config, Some(log_tx.clone())).await?;

    let _ = log_tx.send(LogEntry::new(
        format!(
            "Initializing state for {} wallet(s)...",
            app_config.bot.wallets_to_track.len()
        ),
        LogLevel::Info,
    ));

    let mut wallet_positions: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();

    // ---- initialization ----
    for wallet in &app_config.bot.wallets_to_track {
        if let Ok(positions) =
            fetch_positions_for_wallet(wallet, &app_config.site.data_api_base.clone()).await
        {
            wallet_positions.insert(wallet.clone(), positions.clone());
            let _ = log_tx.send(LogEntry::new(
                format!(
                    "Initialized {}... with {} position(s)",
                    &wallet[..8],
                    positions.len()
                ),
                LogLevel::Info,
            ));
        }
    }

    let _ = log_tx.send(LogEntry::new(
        "Copy trader loop started.".to_string(),
        LogLevel::Info,
    ));

    loop {
        // ---- stop check ----
        for wallet in &app_config.bot.wallets_to_track {
            match fetch_positions_for_wallet(wallet, &app_config.site.data_api_base.clone()).await {
                Ok(current_positions) => {
                    let previous_positions =
                        wallet_positions.get(wallet).cloned().unwrap_or_default();

                    let changes =
                        detect_position_changes(previous_positions, current_positions.clone())
                            .await;

                    // Create semaphore to limit concurrent trade executions
                    // Default to 5 concurrent workers, but can be configured
                    let max_concurrent_trades = changes.len().min(8) as usize; // Use rate_limit as max workers, cap at 10
                    let trade_semaphore = Arc::new(Semaphore::new(max_concurrent_trades));

                    if !changes.is_empty() {
                        for change in &changes {
                            let _ = log_tx.send(LogEntry::new(
                                format!(
                                    "Detected {} for {}: {} shares of {}",
                                    change["type"],
                                    &wallet[..8],
                                    change["size"],
                                    change["title"]
                                ),
                                LogLevel::Info,
                            ));
                        }

                        // Execute copy trades with controlled concurrency using semaphore
                        // This limits the number of concurrent trades to avoid overwhelming the API
                        let copy_trader_clone = copy_trader.clone();
                        let log_tx_clone = log_tx.clone();

                        let futures = changes
                            .into_iter()
                            .map(|change| {
                                let semaphore = trade_semaphore.clone();
                                let mut copy_trader = copy_trader_clone.clone();
                                let log_tx = log_tx_clone.clone();
                                let change_clone = change.clone();

                                tokio::spawn(async move {
                                    // Acquire permit before executing trade (limits concurrency)
                                    let permit = match semaphore.acquire().await {
                                        Ok(p) => p,
                                        Err(e) => {
                                            let _ = log_tx.send(LogEntry::new(
                                                format!("Failed to acquire semaphore permit: {}", e),
                                                LogLevel::Error,
                                            ));
                                            return;
                                        }
                                    };

                                    // Execute the trade and handle the result
                                    match copy_trader.execute_trade(&change_clone, None).await {
                                        Some(_) => {
                                            // Trade executed successfully (logs are handled inside execute_trade)
                                        }
                                        None => {
                                            let title = change_clone.get("title").map(|s| s.as_str()).unwrap_or("unknown");
                                            let _ = log_tx.send(LogEntry::new(
                                                format!("Trade execution returned None for {}", title),
                                                LogLevel::Warning,
                                            ));
                                        }
                                    }

                                    // Permit is automatically released when dropped
                                    drop(permit);
                                })
                            })
                            .collect::<Vec<_>>();

                        // Wait for all trades to complete and log any task panics
                        for future in futures {
                            match future.await {
                                Ok(_) => {
                                    // Task completed successfully
                                }
                                Err(e) => {
                                    let _ = log_tx.send(LogEntry::new(
                                        format!("Trade execution task panicked: {}", e),
                                        LogLevel::Error,
                                    ));
                                }
                            }
                        }
                    }

                    wallet_positions.insert(wallet.clone(), current_positions.clone());
                }
                Err(_) => continue,
            }
        }

        tokio::time::sleep(Duration::from_secs(app_config.trading.poll_interval)).await;
    }
    // Note: The loop above runs indefinitely, so code below is unreachable
    // If you need shutdown handling, add a break condition or signal handler above
}

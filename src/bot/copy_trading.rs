// ============================================================================
// Copy Trading Bot
// ============================================================================

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Duration;

use crate::{
    PreparedCreds,
    config::AppConfig,
    models::ResubmitRequest,
    service::{
        orders::OrderEngine,
        processor::{resubmit_worker, run_ws_loop, start_order_processing_worker},
    },
    ui::components::logs::{LogEntry, LogLevel},
    utils::client::create_authenticated_clob_client,
};

pub async fn run_bot(app_config: AppConfig, log_tx: mpsc::UnboundedSender<LogEntry>) -> Result<()> {
    let (client, creds) = create_authenticated_clob_client(
        app_config.site.clob_api_base.clone(),
        app_config.bot.private_key.clone(),
        app_config.bot.funder_address.clone(),
    )
    .await?;

    let _ = log_tx.send(LogEntry {
        message: "Creating authenticated client...".to_string(),
        level: LogLevel::Info,
    });

    let prepared_creds: PreparedCreds = PreparedCreds::from_api_creds(&creds)?;
    let risk_config = app_config.risk_guard_config();

    let _ = log_tx.send(LogEntry {
        message: "Initializing order processing channels...".to_string(),
        level: LogLevel::Info,
    });

    let (order_tx, order_rx) = mpsc::channel(1024);
    let (resubmit_tx, resubmit_rx) = mpsc::unbounded_channel::<ResubmitRequest>();

    let client_arc = Arc::new(client);
    let creds_arc = Arc::new(prepared_creds.clone());
    let private_key_arc = Arc::new(app_config.bot.private_key.clone());
    let funder_address_arc = Arc::new(app_config.bot.funder_address.clone());

    let _ = log_tx.send(LogEntry {
        message: "Starting order processing worker...".to_string(),
        level: LogLevel::Info,
    });

    start_order_processing_worker(
        order_rx,
        client_arc.clone(),
        app_config.site.clob_api_base.as_str(),
        private_key_arc.clone(),
        funder_address_arc.clone(),
        app_config.bot.enable_trading,
        app_config.bot.mock_trading,
        risk_config,
        resubmit_tx.clone(),
    );

    let _ = log_tx.send(LogEntry {
        message: "Starting resubmit worker...".to_string(),
        level: LogLevel::Info,
    });

    tokio::spawn(resubmit_worker(resubmit_rx, client_arc, creds_arc));

    let order_engine = OrderEngine {
        tx: order_tx.clone(),
        resubmit_tx: resubmit_tx.clone(),
        enable_trading: app_config.bot.enable_trading,
    };

    let _ = log_tx.send(LogEntry {
        message: format!(
            "Connecting to WebSocket (Gamma API: {})...",
            app_config.site.gamma_api_base
        ),
        level: LogLevel::Info,
    });

    let mut backoff_secs = 1u64;
    let max_backoff_secs = 60u64;
    let mut consecutive_failures = 0u32;

    loop {
        let _ = log_tx.send(LogEntry {
            message: "Establishing WebSocket connection...".to_string(),
            level: LogLevel::Info,
        });

        match run_ws_loop(
            &order_engine,
            &app_config.site.gamma_api_base,
            &app_config.site.clob_api_base,
            &app_config.site.clob_wss_url,
        )
        .await
        {
            Ok(_) => {
                // Connection closed normally, reset backoff
                backoff_secs = 1;
                consecutive_failures = 0;
                let _ = log_tx.send(LogEntry {
                    message: "WebSocket connection closed. Reconnecting...".to_string(),
                    level: LogLevel::Warning,
                });
            }
            Err(e) => {
                consecutive_failures += 1;

                // Categorize errors for better handling
                let error_msg = e.to_string();
                let is_tls_error = error_msg.contains("tls")
                    || error_msg.contains("TLS")
                    || error_msg.contains("handshake")
                    || error_msg.contains("close_notify");
                let is_connection_error = error_msg.contains("reset")
                    || error_msg.contains("refused")
                    || error_msg.contains("timeout")
                    || error_msg.contains("Connection");

                // Use exponential backoff with jitter
                let delay = if is_tls_error || is_connection_error {
                    // For connection errors, use longer backoff
                    backoff_secs.min(max_backoff_secs)
                } else {
                    // For other errors, use shorter backoff
                    (backoff_secs / 2).max(1)
                };

                // Add jitter (±20%) to avoid thundering herd
                let jitter = (delay as f64 * 0.2) * (rand::random::<f64>() * 2.0 - 1.0);
                let final_delay = (delay as f64 + jitter).max(1.0) as u64;

                let _ = log_tx.send(LogEntry {
                    message: format!(
                        "WebSocket error (attempt {}): {}. Reconnecting in {}s...",
                        consecutive_failures, e, final_delay
                    ),
                    level: LogLevel::Error,
                });

                tokio::time::sleep(Duration::from_secs(final_delay)).await;

                // Exponential backoff: double the delay, capped at max
                backoff_secs = (backoff_secs * 2).min(max_backoff_secs);
            }
        }
    }
}

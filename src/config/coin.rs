use tokio::sync::mpsc;

use crate::ui::components::logs::{LogEntry, LogLevel};

// Supported coins for 15-min markets (FYI: these are the only ones we track)
pub const AVAILABLE_COINS: &[&str] = &["BTC", "ETH", "SOL", "XRP"];

pub fn display_coin_selection(selected_index: usize, log_tx: mpsc::UnboundedSender<LogEntry>) {
    for (index, coin) in AVAILABLE_COINS.iter().enumerate() {
        let is_selected = index == selected_index;
        let prefix = if is_selected { "> " } else { "  " };
        let _ = log_tx.send(LogEntry::new(format!("{}{}", prefix, coin), LogLevel::Info));
    }
}

pub fn get_available_coins() -> Vec<&'static str> {
    AVAILABLE_COINS.to_vec()
}

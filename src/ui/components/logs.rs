use ratatui::style::Color;
use chrono::{DateTime, Local};

/// Log entry with timestamp and level
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub message: String,
    pub level: LogLevel,
    pub timestamp: DateTime<Local>,
}

impl LogEntry {
    /// Create a new log entry with the current local timestamps
    pub fn new(message: String, level: LogLevel) -> Self {
        Self {
            message,
            level,
            timestamp: Local::now(),
        }
    }

    /// Format the timestamp as a string
    pub fn formatted_timestamp(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
}

impl LogLevel {
    pub fn color(&self) -> Color {
        match self {
            LogLevel::Info => Color::White,
            LogLevel::Warning => Color::Yellow,
            LogLevel::Error => Color::Red,
            LogLevel::Success => Color::Green,
        }
    }
}
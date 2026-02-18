use ratatui::style::Color;

/// Log entry with timestamp and level
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub message: String,
    pub level: LogLevel,
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
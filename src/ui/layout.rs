use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarState,
};
use ratatui::{Frame, Terminal};
use std::collections::VecDeque;
use std::io;
use tokio::sync::mpsc;

use crate::ui::components::logs::{LogLevel, LogEntry};

/// Bot selection options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotType {
    CopyTrading,
    Arbitrage,
    Sniper,
}

impl BotType {
    pub fn name(&self) -> &'static str {
        match self {
            BotType::CopyTrading => "Copy Trading Bot",
            BotType::Arbitrage => "Arbitrage Bot",
            BotType::Sniper => "Sniper Bot",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BotType::CopyTrading => "Automatically copy trades from selected traders",
            BotType::Arbitrage => "Find and exploit price differences across markets",
            BotType::Sniper => "Execute trades at optimal prices instantly",
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            BotType::CopyTrading => true,
            BotType::Arbitrage => false,
            BotType::Sniper => false,
        }
    }

    pub fn status(&self) -> &'static str {
        if self.is_enabled() {
            "✓ Available"
        } else {
            "✗ Coming Soon"
        }
    }
}

/// UI state for bot selection
pub struct BotSelectionUI {
    pub selected: usize,
    pub bots: Vec<BotType>,
}

impl BotSelectionUI {
    pub fn new() -> Self {
        Self {
            selected: 0,
            bots: vec![BotType::CopyTrading, BotType::Arbitrage, BotType::Sniper],
        }
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.bots.len();
    }

    pub fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.bots.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn selected_bot(&self) -> BotType {
        self.bots[self.selected]
    }
}

/// UI state for running bot with logs
pub struct RunningBotUI {
    pub bot_name: String,
    pub logs: VecDeque<LogEntry>,
    pub scroll_offset: usize,
    pub max_logs: usize,
}

impl RunningBotUI {
    pub fn new(bot_name: String) -> Self {
        Self {
            bot_name,
            logs: VecDeque::new(),
            scroll_offset: 0,
            max_logs: 1000, // Keep last 1000 logs
        }
    }

    pub fn add_log(&mut self, entry: LogEntry) {
        self.logs.push_back(entry);
        if self.logs.len() > self.max_logs {
            self.logs.pop_front();
        }
        // Auto-scroll to bottom
        self.scroll_offset = 0;
    }

    pub fn scroll_up(&mut self, amount: usize) {
        let max_offset = self.logs.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }
}

/// Render the main UI
pub fn render_selection_ui(f: &mut Frame, ui: &BotSelectionUI) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Bot list
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new("🚀 Polymarket Trading Toolkits")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(header, chunks[0]);

    // Bot selection list
    let items: Vec<ListItem> = ui
        .bots
        .iter()
        .enumerate()
        .map(|(idx, bot)| {
            let is_selected = idx == ui.selected;
            let is_enabled = bot.is_enabled();

            let style = if is_selected {
                Style::default()
                    .fg(if is_enabled {
                        Color::Green
                    } else {
                        Color::Yellow
                    })
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(if is_enabled {
                    Color::White
                } else {
                    Color::DarkGray
                })
            };

            let prefix = if is_selected {
                if is_enabled { "▶ " } else { "▶ " }
            } else {
                if is_enabled { "  " } else { "  " }
            };

            let status_span = Span::styled(
                bot.status(),
                Style::default().fg(if is_enabled {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            );

            ListItem::new(vec![Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(bot.name(), style),
                Span::raw(" - "),
                Span::styled(bot.description(), Style::default().fg(Color::Gray)),
                Span::raw(" "),
                status_span,
            ])])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Trading Bot")
                .style(Style::default().fg(Color::White)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(ui.selected));
    f.render_stateful_widget(list, chunks[1], &mut list_state);

    // Footer with instructions
    let instructions = if ui.selected_bot().is_enabled() {
        "Press Enter to start | ↑/↓ to navigate | Q to quit"
    } else {
        "↑/↓ to navigate | Q to quit | This bot is coming soon"
    };

    let footer = Paragraph::new(instructions)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(footer, chunks[2]);
}

/// Render the running bot UI with logs
pub fn render_running_ui(f: &mut Frame, ui: &RunningBotUI) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Logs area
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header with bot name
    let header_text = format!("🚀 {} - Running", ui.bot_name);
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(header, chunks[0]);

    // Logs area
    let log_lines: Vec<Line> = ui
        .logs
        .iter()
        .rev()
        .skip(ui.scroll_offset)
        .take(chunks[1].height as usize - 2)
        .rev()
        .map(|entry| {
            let prefix = match entry.level {
                LogLevel::Info => "[INFO]",
                LogLevel::Warning => "[WARN]",
                LogLevel::Error => "[ERROR]",
                LogLevel::Success => "[OK]",
            };
            Line::from(vec![
                Span::styled(
                    entry.formatted_timestamp(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" "),
                Span::styled(
                    prefix,
                    Style::default()
                        .fg(entry.level.color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(entry.message.clone(), Style::default().fg(entry.level.color())),
            ])
            })
        .collect();

    let log_widget = Paragraph::new(log_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Logs")
                .style(Style::default().fg(Color::White)),
        )
        .scroll((ui.scroll_offset as u16, 0));

    f.render_widget(log_widget, chunks[1]);

    // Scrollbar
    let total_logs = ui.logs.len();
    let visible_logs = chunks[1].height as usize - 2;
    if total_logs > visible_logs {
        let scrollbar = Scrollbar::default()
            .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::new(total_logs)
            .position(ui.scroll_offset)
            .content_length(visible_logs);

        f.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);
    }

    // Footer with instructions
    let footer = Paragraph::new("Press Q to quit | ↑/↓ to scroll logs")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        );
    f.render_widget(footer, chunks[2]);
}

/// Run the UI and return the selected bot
pub fn run_selection_ui() -> Result<Option<BotType>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut ui = BotSelectionUI::new();
    let mut result = None;

    loop {
        terminal.draw(|f| render_selection_ui(f, &ui))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Up => {
                        ui.previous();
                    }
                    KeyCode::Down => {
                        ui.next();
                    }
                    KeyCode::Enter => {
                        if ui.selected_bot().is_enabled() {
                            result = Some(ui.selected_bot());
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(result)
}

/// Run the bot UI with log display
pub async fn run_bot_ui<F>(
    bot_name: String,
    mut log_rx: mpsc::UnboundedReceiver<LogEntry>,
    bot_task: F,
) -> Result<()>
where
    F: std::future::Future<Output = Result<()>> + Send + 'static,
{
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut ui = RunningBotUI::new(bot_name.clone());

    // Spawn bot task
    let bot_handle = tokio::spawn(bot_task);

    // Event loop
    let mut should_quit = false;

    loop {
        // Render UI
        terminal.draw(|f| render_running_ui(f, &ui))?;

        // Handle keyboard events (non-blocking poll)
        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            should_quit = true;
                        }
                        KeyCode::Up => {
                            ui.scroll_up(5);
                        }
                        KeyCode::Down => {
                            ui.scroll_down(5);
                        }
                        KeyCode::PageUp => {
                            ui.scroll_up(20);
                        }
                        KeyCode::PageDown => {
                            ui.scroll_down(20);
                        }
                        KeyCode::Home => {
                            ui.scroll_to_bottom();
                        }
                        KeyCode::End => {
                            ui.scroll_offset = ui.logs.len().saturating_sub(1);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Handle log updates (non-blocking)
        while let Ok(entry) = log_rx.try_recv() {
            ui.add_log(entry);
        }

        // Small delay to prevent CPU spinning
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        if should_quit {
            // Send shutdown message
            let _ = ui.logs.push_back(LogEntry::new(
                "Shutting down bot...".to_string(),
                LogLevel::Warning,
            ));
            terminal.draw(|f| render_running_ui(f, &ui))?;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            break;
        }

        // Check if bot task completed (non-blocking)
        if bot_handle.is_finished() {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

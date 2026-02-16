use std::io;
use anyhow::Result;
use crossterm::{event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}};
use ratatui::{
    DefaultTerminal, Frame, Terminal, buffer::Buffer, layout::{Alignment, Constraint, Direction, Layout, Rect}, prelude::CrosstermBackend, style::{Color, Modifier, Style, Stylize}, symbols::border, text::{Line, Text}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget}
};

use polymarket_client_sdk::clob::Client;

#[derive(Debug, Default)]
pub struct App {
    items: Vec<&'static str>,
    state: ListState,
    selected: Option<String>,
    exit: bool,
}

impl App {
    fn new() -> Self {
        let items = vec!["Start node", "Stop node", "Restart", "Status", "Exit"];
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            items,
            state,
            selected: None,
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Down => self.next(),
            KeyCode::Up => self.previous(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Next ".into(),
            "<Down>".blue().bold(),
            " Previous ".into(),
            "<Up>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let selected_text = Text::from(vec![Line::from(vec![
            "Selected: ".into(),
                self.selected.as_ref().unwrap_or(&"None".to_string()).clone().yellow(),
        ])]);

        Paragraph::new(selected_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::default();

    client.create_api_key(signer, nonce);

    let ok = client.ok().await?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::Enter => {
                    if let Some(i) = app.state.selected() {
                        app.selected = Some(app.items[i].to_string());
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|i| ListItem::new(*i))
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.state);

    let msg = match &app.selected {
        Some(s) => format!("Selected: {s}"),
        None => "Press Enter to select".into(),
    };

    let paragraph = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Output"));

    f.render_widget(paragraph, chunks[1]);
}

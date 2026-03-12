use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;

pub enum KeyAction {
    Up,
    Down,
    Enter,
    Exit,
    None,
}

pub struct KeyboardHandler {
    enabled: bool,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self { enabled: false }
    }

    pub fn enable(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        self.enabled = true;
        Ok(())
    }

    pub fn disable(&mut self) -> io::Result<()> {
        if self.enabled {
            disable_raw_mode()?;
            self.enabled = false;
        }
        Ok(())
    }

    pub fn read_key(&self) -> io::Result<KeyAction> {
        if !self.enabled {
            return Ok(KeyAction::None);
        }

        match event::read()? {
            Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) => match code {
                KeyCode::Up => Ok(KeyAction::Up),
                KeyCode::Down => Ok(KeyAction::Down),
                KeyCode::Enter => Ok(KeyAction::Enter),
                KeyCode::Char('c') => {
                    // Check for Ctrl+C
                    if crossterm::event::read()? == Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        kind: KeyEventKind::Press,
                        modifiers: event::KeyModifiers::CONTROL,
                        state: event::KeyEventState::all(),
                    }) {
                        Ok(KeyAction::Exit)
                    } else {
                        Ok(KeyAction::None)
                    }
                }
                _ => Ok(KeyAction::None),
            },
            _ => Ok(KeyAction::None),
        }
    }
}

impl Drop for KeyboardHandler {
    fn drop(&mut self) {
        let _ = self.disable();
    }
}
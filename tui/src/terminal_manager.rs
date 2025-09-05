//! Terminal setup and cleanup management

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{DisableMouseCapture, EnableMouseCapture},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io};

/// Terminal manager for handling terminal setup and cleanup
pub struct TerminalManager;

impl TerminalManager {
    /// Set up the terminal for TUI mode
    pub fn setup() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    /// Clean up terminal state and restore normal mode
    pub fn cleanup<B: Backend + std::io::Write>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    /// Perform emergency cleanup (best effort, ignores errors)
    pub fn emergency_cleanup() {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

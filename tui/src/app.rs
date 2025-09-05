//! Main TUI application state and event handling

use crate::{create_confluence_client, screens::Screen, ui::Ui};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nix_rust_template::ConfluenceClient;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io, time::Duration};

/// Main application state
pub struct App {
    /// Should the application exit?
    pub should_quit: bool,
    /// Current active screen
    pub current_screen: Screen,
    /// Confluence client for API operations
    pub confluence_client: ConfluenceClient,
    /// UI state handler
    pub ui: Ui,
}

impl App {
    /// Create a new App instance
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let confluence_client = create_confluence_client()?;
        let ui = Ui::new();

        Ok(Self {
            should_quit: false,
            current_screen: Screen::MainMenu,
            confluence_client,
            ui,
        })
    }

    /// Run the TUI application
    pub fn run(mut self) -> Result<(), Box<dyn Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main application loop
        let result = self.run_app(&mut terminal);

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    /// Main application event loop
    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
        loop {
            // Draw UI
            terminal.draw(|f| self.ui.draw(f, &self))?;

            // Handle events with timeout
            if event::poll(Duration::from_millis(100))? {
                self.handle_event(event::read()?)?;
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    /// Handle incoming events
    fn handle_event(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
        match event {
            Event::Key(KeyEvent { code, .. }) => {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_quit = true;
                    }
                    _ => {
                        // Handle key event based on current screen
                        match self.current_screen {
                            Screen::MainMenu => {
                                match code {
                                    KeyCode::Char('1') => {
                                        self.switch_screen(Screen::CqlBuilder);
                                    }
                                    KeyCode::Char('2') => {
                                        self.switch_screen(Screen::PageBrowser);
                                    }
                                    KeyCode::Char('3') => {
                                        self.switch_screen(Screen::LabelManager);
                                    }
                                    KeyCode::Char('h') => {
                                        self.switch_screen(Screen::Help);
                                    }
                                    _ => {}
                                }
                            }
                            Screen::CqlBuilder => {
                                match code {
                                    KeyCode::Backspace => {
                                        self.switch_screen(Screen::MainMenu);
                                    }
                                    KeyCode::Enter => {
                                        self.switch_screen(Screen::PageBrowser);
                                    }
                                    _ => {
                                        // TODO: Handle text input for CQL query
                                    }
                                }
                            }
                            Screen::PageBrowser => {
                                match code {
                                    KeyCode::Backspace => {
                                        self.switch_screen(Screen::MainMenu);
                                    }
                                    KeyCode::Enter => {
                                        self.switch_screen(Screen::LabelManager);
                                    }
                                    KeyCode::Up | KeyCode::Down => {
                                        // TODO: Handle navigation
                                    }
                                    _ => {}
                                }
                            }
                            Screen::LabelManager => {
                                match code {
                                    KeyCode::Backspace => {
                                        self.switch_screen(Screen::PageBrowser);
                                    }
                                    KeyCode::Char('a') => {
                                        // TODO: Add label mode
                                    }
                                    KeyCode::Char('d') => {
                                        // TODO: Delete label mode
                                    }
                                    KeyCode::Char('u') => {
                                        // TODO: Update label mode
                                    }
                                    _ => {}
                                }
                            }
                            Screen::Help => {
                                match code {
                                    KeyCode::Backspace | KeyCode::Esc => {
                                        self.switch_screen(Screen::MainMenu);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Switch to a different screen
    pub fn switch_screen(&mut self, screen: Screen) {
        self.current_screen = screen;
    }
}

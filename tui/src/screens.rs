//! Different screens/views for the TUI application

use crate::app::App;
use crossterm::event::KeyCode;
use std::error::Error;

/// Enum representing different screens in the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    /// Tree navigation screen for selecting domain/product/project
    TreeNavigation,
    /// Main menu screen (legacy)
    MainMenu,
    /// CQL query builder screen
    CqlBuilder,
    /// Page browser screen showing results
    PageBrowser,
    /// Label management screen
    LabelManager,
    /// Help screen
    Help,
    /// Command execution screen for running ctag commands
    CommandExecution,
}

impl Screen {
    /// Handle key events for the current screen
    pub fn handle_key_event(
        &mut self,
        app: &mut App,
        key_code: KeyCode,
    ) -> Result<(), Box<dyn Error>> {
        match self {
            Screen::TreeNavigation => self.handle_tree_navigation_keys(app, key_code),
            Screen::CommandExecution => self.handle_command_execution_keys(app, key_code),
            Screen::MainMenu => self.handle_main_menu_keys(app, key_code),
            Screen::CqlBuilder => self.handle_cql_builder_keys(app, key_code),
            Screen::PageBrowser => self.handle_page_browser_keys(app, key_code),
            Screen::LabelManager => self.handle_label_manager_keys(app, key_code),
            Screen::Help => self.handle_help_keys(app, key_code),
        }
    }

    /// Handle keys for main menu screen
    fn handle_main_menu_keys(
        &mut self,
        app: &mut App,
        key_code: KeyCode,
    ) -> Result<(), Box<dyn Error>> {
        match key_code {
            KeyCode::Char('1') => {
                app.switch_screen(Screen::CqlBuilder);
            }
            KeyCode::Char('2') => {
                app.switch_screen(Screen::PageBrowser);
            }
            KeyCode::Char('3') => {
                app.switch_screen(Screen::LabelManager);
            }
            KeyCode::Char('h') => {
                app.switch_screen(Screen::Help);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys for CQL builder screen
    fn handle_cql_builder_keys(
        &mut self,
        app: &mut App,
        key_code: KeyCode,
    ) -> Result<(), Box<dyn Error>> {
        match key_code {
            KeyCode::Backspace => {
                app.switch_screen(Screen::MainMenu);
            }
            KeyCode::Enter => {
                // Execute CQL query and switch to page browser
                app.switch_screen(Screen::PageBrowser);
            }
            _ => {
                // Handle text input for CQL query
                // TODO: Implement CQL input handling
            }
        }
        Ok(())
    }

    /// Handle keys for page browser screen
    fn handle_page_browser_keys(
        &mut self,
        app: &mut App,
        key_code: KeyCode,
    ) -> Result<(), Box<dyn Error>> {
        match key_code {
            KeyCode::Backspace => {
                app.switch_screen(Screen::MainMenu);
            }
            KeyCode::Enter => {
                // Open selected page in label manager
                app.switch_screen(Screen::LabelManager);
            }
            KeyCode::Up => {
                // Move selection up
                // TODO: Implement navigation
            }
            KeyCode::Down => {
                // Move selection down
                // TODO: Implement navigation
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys for label manager screen
    fn handle_label_manager_keys(
        &mut self,
        app: &mut App,
        key_code: KeyCode,
    ) -> Result<(), Box<dyn Error>> {
        match key_code {
            KeyCode::Backspace => {
                app.switch_screen(Screen::PageBrowser);
            }
            KeyCode::Char('a') => {
                // Add label mode
                // TODO: Implement label adding
            }
            KeyCode::Char('d') => {
                // Delete label mode
                // TODO: Implement label deletion
            }
            KeyCode::Char('u') => {
                // Update label mode
                // TODO: Implement label updating
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys for help screen
    fn handle_help_keys(&mut self, app: &mut App, key_code: KeyCode) -> Result<(), Box<dyn Error>> {
        match key_code {
            KeyCode::Backspace | KeyCode::Esc => {
                app.switch_screen(Screen::MainMenu);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys for tree navigation screen
    fn handle_tree_navigation_keys(
        &mut self,
        app: &mut App,
        key_code: KeyCode,
    ) -> Result<(), Box<dyn Error>> {
        match key_code {
            KeyCode::Enter => {
                // TODO: Select current tree node
            }
            KeyCode::Up => {
                // TODO: Move up in tree
            }
            KeyCode::Down => {
                // TODO: Move down in tree
            }
            KeyCode::Right => {
                // TODO: Expand node or navigate to commands
            }
            KeyCode::Left => {
                // TODO: Collapse node
            }
            KeyCode::Char('c') => {
                // Switch to command execution for ctag
                if app.get_navigation_context().is_complete() {
                    app.switch_screen(Screen::CommandExecution);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keys for command execution screen
    fn handle_command_execution_keys(
        &mut self,
        app: &mut App,
        key_code: KeyCode,
    ) -> Result<(), Box<dyn Error>> {
        match key_code {
            KeyCode::Backspace | KeyCode::Esc => {
                app.switch_screen(Screen::TreeNavigation);
            }
            KeyCode::Enter => {
                // TODO: Execute ctag command
            }
            _ => {
                // TODO: Handle command input
            }
        }
        Ok(())
    }
}

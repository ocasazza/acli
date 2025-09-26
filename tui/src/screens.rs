//! Different screens/views for the TUI application

use crate::app::App;
use crossterm::event::KeyCode;
use std::error::Error;

/// Enum representing different screens in the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    /// Tree navigation screen for selecting domain/product/project
    TreeNavigation,
    /// Page selection screen for building label buffer
    PageSelection,
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
            Screen::PageSelection => self.handle_page_selection_keys(app, key_code),
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
        if app.is_search_mode() {
            // Handle search mode input
            match key_code {
                KeyCode::Esc => {
                    app.search_manager.exit_search_mode(&mut app.ui);
                    app.tree_navigation.tree_selection = 0;
                }
                KeyCode::Enter => {
                    // Apply search filter and select item
                    if let Some(original_index) = app
                        .search_manager
                        .get_original_index_for_filtered_item(app.tree_navigation.tree_selection)
                    {
                        app.tree_navigation.tree_selection = original_index;
                        app.tree_navigation
                            .select_current_node_with_parents(app.domain.as_ref())?;
                        app.command_executor
                            .update_context(app.tree_navigation.navigation_context.clone());
                    }
                    app.search_manager.exit_search_mode(&mut app.ui);
                }
                KeyCode::Backspace => {
                    let tree_items = app.tree_navigation.get_tree_items();
                    app.tree_navigation.tree_selection =
                        app.search_manager.remove_from_query(&tree_items);
                }
                KeyCode::Char(c) => {
                    let tree_items = app.tree_navigation.get_tree_items();
                    app.tree_navigation.tree_selection =
                        app.search_manager.add_to_query(c, &tree_items);
                }
                KeyCode::Up => {
                    if app.search_manager.filtered_tree_items.is_some() {
                        if app.tree_navigation.tree_selection > 0 {
                            app.tree_navigation.tree_selection -= 1;
                        }
                    } else {
                        app.tree_navigation.move_selection_up();
                    }
                }
                KeyCode::Down => {
                    if let Some(ref filtered_items) = app.search_manager.filtered_tree_items {
                        if app.tree_navigation.tree_selection < filtered_items.len().saturating_sub(1) {
                            app.tree_navigation.tree_selection += 1;
                        }
                    } else {
                        app.tree_navigation.move_selection_down();
                    }
                }
                _ => {}
            }
        } else {
            // Handle normal navigation
            match key_code {
                KeyCode::Enter => {
                    app.tree_navigation
                        .select_current_node(app.domain.as_ref())?;
                    app.command_executor
                        .update_context(app.tree_navigation.navigation_context.clone());
                }
                KeyCode::Up => {
                    app.tree_navigation.move_selection_up();
                }
                KeyCode::Down => {
                    app.tree_navigation.move_selection_down();
                }
                KeyCode::Right => {
                    app.tree_navigation.expand_current_node();
                }
                KeyCode::Left => {
                    app.tree_navigation.collapse_current_node();
                }
                KeyCode::PageUp => {
                    app.tree_navigation.page_up();
                }
                KeyCode::PageDown => {
                    app.tree_navigation.page_down();
                }
                KeyCode::Char('/') => {
                    app.search_manager.enter_search_mode(&mut app.ui);
                }
                KeyCode::Char('c') => {
                    // Switch to page selection for ctag
                    if app.get_navigation_context().is_complete() {
                        app.switch_screen(Screen::PageSelection);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle keys for page selection screen
    fn handle_page_selection_keys(
        &mut self,
        app: &mut App,
        key_code: KeyCode,
    ) -> Result<(), Box<dyn Error>> {
        match key_code {
            KeyCode::Backspace | KeyCode::Esc => {
                app.switch_screen(Screen::TreeNavigation);
            }
            KeyCode::Enter => {
                // Check if current selection is a page that can be expanded (has children to load)
                let tree_items = app.get_page_tree_items();
                let selection_index = app.get_page_tree_selection();

                if let Some(node_path) = app.page_tree_navigation.get_node_path_at_index(selection_index) {
                    // Try to expand the current page node (which will trigger lazy loading if needed)
                    app.expand_current_page_node();

                    // If no children were loaded and it's a leaf node, switch to command execution
                    let tree_items_after = app.get_page_tree_items();
                    if tree_items.len() == tree_items_after.len() {
                        // No expansion happened, switch to commands
                        app.switch_screen(Screen::CommandExecution);
                    }
                } else {
                    // Fallback to command execution
                    app.switch_screen(Screen::CommandExecution);
                }
            }
            KeyCode::Up => {
                app.move_page_selection_up();
            }
            KeyCode::Down => {
                app.move_page_selection_down();
            }
            KeyCode::Right => {
                app.expand_current_page_node();
            }
            KeyCode::Left => {
                app.collapse_current_page_node();
            }
            KeyCode::Char(' ') => {
                // Toggle page selection for label buffer
                app.toggle_page_selection()?;
            }
            KeyCode::Char('/') => {
                app.enter_page_search_mode();
            }
            KeyCode::Char('c') => {
                // Clear label buffer
                app.clear_label_buffer();
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
        use crate::command::{AvailableCommand, CommandInputMode};

        match key_code {
            KeyCode::Backspace | KeyCode::Esc => {
                app.switch_screen(Screen::PageSelection);
            }
            KeyCode::Enter => {
                match app.command_input.mode {
                    CommandInputMode::SelectingCommand => {
                        // Select the current command
                        let available_commands = app.command_executor.get_available_commands();
                        if let Some(command) = available_commands.get(app.command_selection) {
                            app.command_input.set_command(command.clone());
                        }
                    }
                    CommandInputMode::TypingArgs => {
                        // Execute the command with current args
                        Self::execute_selected_command(app)?;
                    }
                    CommandInputMode::Ready => {
                        // Execute the command
                        Self::execute_selected_command(app)?;
                    }
                }
            }
            KeyCode::Up => {
                if let Some(ref mut tree_manager) = app.command_result_tree {
                    // Navigate up in tree results
                    tree_manager.move_selection_up();
                } else if !app.command_output.is_empty() {
                    // Scroll up in the output
                    app.command_output_scroll = app.command_output_scroll.saturating_sub(1);
                } else if app.command_input.mode == CommandInputMode::SelectingCommand
                    && app.command_selection > 0
                {
                    app.command_selection -= 1;
                }
            }
            KeyCode::Down => {
                if let Some(ref mut tree_manager) = app.command_result_tree {
                    // Navigate down in tree results
                    tree_manager.move_selection_down();
                } else if !app.command_output.is_empty() {
                    // Calculate proper scroll bounds based on visible area
                    let visible_lines = 6;
                    let max_scroll = app.command_output.len().saturating_sub(visible_lines);
                    if app.command_output_scroll < max_scroll {
                        app.command_output_scroll += 1;
                    }
                } else if app.command_input.mode == CommandInputMode::SelectingCommand {
                    let available_commands = app.command_executor.get_available_commands();
                    if app.command_selection < available_commands.len().saturating_sub(1) {
                        app.command_selection += 1;
                    }
                }
            }
            KeyCode::Left => {
                if let Some(ref mut tree_manager) = app.command_result_tree {
                    // Collapse current node in tree results
                    tree_manager.collapse_current_node();
                } else if matches!(app.command_input.mode, CommandInputMode::TypingArgs) {
                    app.command_input.move_cursor_left();
                }
            }
            KeyCode::Right => {
                if let Some(ref mut tree_manager) = app.command_result_tree {
                    // Expand current node in tree results
                    tree_manager.expand_current_node();
                } else if matches!(app.command_input.mode, CommandInputMode::TypingArgs) {
                    app.command_input.move_cursor_right();
                }
            }
            KeyCode::Delete => {
                if matches!(app.command_input.mode, CommandInputMode::TypingArgs) {
                    app.command_input.delete_char();
                }
            }
            KeyCode::Char(c) => {
                match app.command_input.mode {
                    CommandInputMode::TypingArgs => {
                        app.command_input.insert_char(c);
                    }
                    CommandInputMode::SelectingCommand => {
                        // Quick selection by first letter
                        let available_commands = app.command_executor.get_available_commands();
                        for (i, command) in available_commands.iter().enumerate() {
                            let AvailableCommand::Ctag { operation, .. } = command;
                            let first_char = operation
                                .as_str()
                                .chars()
                                .next()
                                .unwrap_or(' ')
                                .to_ascii_lowercase();
                            if c.to_ascii_lowercase() == first_char {
                                app.command_selection = i;
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Execute the currently selected command
    fn execute_selected_command(app: &mut App) -> Result<(), Box<dyn Error>> {
        use crate::command::{AvailableCommand, TuiCommand};

        if let Some(AvailableCommand::Ctag { operation, .. }) = &app.command_input.selected_command {
            // Parse additional arguments from command input
            let args: Vec<String> = if app.command_input.text.trim().is_empty() {
                Vec::new()
            } else {
                app.command_input
                    .text
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect()
            };

            let command = TuiCommand {
                name: "ctag".to_string(),
                operation: operation.as_str().to_string(),
                args,
                dry_run: false,
            };

            // Get label buffer for command execution
            let label_buffer = app.get_label_buffer();
            let label_buffer_ref = if !label_buffer.is_empty() {
                Some(label_buffer.as_slice())
            } else {
                None
            };

            // Execute the command with label buffer
            match app.command_executor.execute_command(command, label_buffer_ref) {
                Ok(result) => {
                    let (output, status_msg) = if result.success {
                        (
                            result.stdout,
                            format!("Command executed successfully: {}", result.command),
                        )
                    } else {
                        (result.stderr, format!("Command failed: {}", result.command))
                    };

                    app.command_output = output.lines().map(|s| s.to_string()).collect();
                    app.command_output_scroll = 0;
                    app.command_result_tree = None;
                    app.ui.set_status(status_msg);
                }
                Err(e) => {
                    let error_msg = format!("Error executing command: {e}");
                    app.command_output = vec![error_msg.clone()];
                    app.command_output_scroll = 0;
                    app.command_result_tree = None;
                    app.ui.set_status(error_msg);
                }
            }
        }

        Ok(())
    }
}

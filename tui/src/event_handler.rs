//! Event handling logic for the TUI application

use crate::{
    app::App,
    command::{AvailableCommand, CommandInputMode, TuiCommand},
    screens::Screen,
};
use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use std::error::Error;

/// Event handler for the TUI application
pub struct EventHandler;

impl EventHandler {
    /// Create a new EventHandler instance
    pub fn new() -> Self {
        Self
    }

    /// Handle incoming events
    pub fn handle_event(app: &mut App, event: Event) -> Result<(), Box<dyn Error>> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => {
                // Handle Ctrl+C
                if code == KeyCode::Char('c')
                    && modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    app.should_quit = true;
                    return Ok(());
                }

                match app.current_screen {
                    Screen::TreeNavigation => {
                        if app.is_search_mode() {
                            Self::handle_search_input(app, code)?;
                        } else {
                            Self::handle_tree_navigation_input(app, code)?;
                        }
                    }
                    Screen::CommandExecution => {
                        Self::handle_command_execution_input(app, code)?;
                    }
                    Screen::MainMenu => {
                        Self::handle_main_menu_input(app, code);
                    }
                    Screen::CqlBuilder => {
                        Self::handle_cql_builder_input(app, code);
                    }
                    Screen::PageBrowser => {
                        Self::handle_page_browser_input(app, code);
                    }
                    Screen::LabelManager => {
                        Self::handle_label_manager_input(app, code);
                    }
                    Screen::Help => {
                        Self::handle_help_input(app, code);
                    }
                }
            }
            Event::Mouse(MouseEvent { kind, .. }) => {
                Self::handle_mouse_event(app, kind)?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle tree navigation input
    fn handle_tree_navigation_input(app: &mut App, code: KeyCode) -> Result<(), Box<dyn Error>> {
        match code {
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
            KeyCode::Char('c') => {
                // Switch to command execution for ctag
                if app.tree_navigation.navigation_context.is_complete() {
                    app.switch_screen(Screen::CommandExecution);
                }
            }
            KeyCode::Char('/') => {
                // Enter search mode
                app.search_manager.enter_search_mode(&mut app.ui);
            }
            KeyCode::PageUp => {
                app.tree_navigation.page_up();
            }
            KeyCode::PageDown => {
                app.tree_navigation.page_down();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle search input
    fn handle_search_input(app: &mut App, code: KeyCode) -> Result<(), Box<dyn Error>> {
        match code {
            KeyCode::Esc => {
                app.search_manager.exit_search_mode(&mut app.ui);
                app.tree_navigation.tree_selection = 0;
            }
            KeyCode::Enter => {
                // When pressing Enter in search mode, we need to:
                // 1. Map the current selection from filtered items back to the full tree using original index
                // 2. Select that item properly
                // 3. Exit search mode
                if let Some(original_index) = app
                    .search_manager
                    .get_original_index_for_filtered_item(app.tree_navigation.tree_selection)
                {
                    // Update tree selection to the correct index in the full tree
                    app.tree_navigation.tree_selection = original_index;

                    // Now select the node properly (this handles parent selection automatically)
                    app.tree_navigation
                        .select_current_node_with_parents(app.domain.as_ref())?;
                    app.command_executor
                        .update_context(app.tree_navigation.navigation_context.clone());
                }
                // Completely exit search mode when a selection is made
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
        Ok(())
    }

    /// Handle mouse events
    fn handle_mouse_event(app: &mut App, kind: MouseEventKind) -> Result<(), Box<dyn Error>> {
        match kind {
            MouseEventKind::ScrollUp => {
                if app.current_screen == Screen::TreeNavigation && !app.is_search_mode() {
                    app.tree_navigation.move_selection_up();
                }
            }
            MouseEventKind::ScrollDown => {
                if app.current_screen == Screen::TreeNavigation && !app.is_search_mode() {
                    app.tree_navigation.move_selection_down();
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle main menu input
    fn handle_main_menu_input(app: &mut App, code: KeyCode) {
        match code {
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
    }

    /// Handle CQL builder input
    fn handle_cql_builder_input(app: &mut App, code: KeyCode) {
        match code {
            KeyCode::Backspace => {
                app.switch_screen(Screen::MainMenu);
            }
            KeyCode::Enter => {
                app.switch_screen(Screen::PageBrowser);
            }
            _ => {
                // TODO: Handle text input for CQL query
            }
        }
    }

    /// Handle page browser input
    fn handle_page_browser_input(app: &mut App, code: KeyCode) {
        match code {
            KeyCode::Backspace => {
                app.switch_screen(Screen::MainMenu);
            }
            KeyCode::Enter => {
                app.switch_screen(Screen::LabelManager);
            }
            KeyCode::Up | KeyCode::Down => {
                // TODO: Handle navigation
            }
            _ => {}
        }
    }

    /// Handle label manager input
    fn handle_label_manager_input(app: &mut App, code: KeyCode) {
        match code {
            KeyCode::Backspace => {
                app.switch_screen(Screen::PageBrowser);
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

    /// Handle help input
    fn handle_help_input(app: &mut App, code: KeyCode) {
        match code {
            KeyCode::Backspace | KeyCode::Esc => {
                app.switch_screen(Screen::MainMenu);
            }
            _ => {}
        }
    }

    /// Handle input for command execution screen
    fn handle_command_execution_input(app: &mut App, code: KeyCode) -> Result<(), Box<dyn Error>> {
        match code {
            KeyCode::Backspace | KeyCode::Esc => {
                app.switch_screen(Screen::TreeNavigation);
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
                if app.command_input.mode == CommandInputMode::SelectingCommand
                    && app.command_selection > 0
                {
                    app.command_selection -= 1;
                }
            }
            KeyCode::Down => {
                if app.command_input.mode == CommandInputMode::SelectingCommand {
                    let available_commands = app.command_executor.get_available_commands();
                    if app.command_selection < available_commands.len().saturating_sub(1) {
                        app.command_selection += 1;
                    }
                }
            }
            KeyCode::Left => {
                if matches!(app.command_input.mode, CommandInputMode::TypingArgs) {
                    app.command_input.move_cursor_left();
                }
            }
            KeyCode::Right => {
                if matches!(app.command_input.mode, CommandInputMode::TypingArgs) {
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

            // Execute the command
            match app.command_executor.execute_command(command) {
                Ok(result) => {
                    let status_msg = if result.success {
                        format!("Command executed successfully: {}", result.command)
                    } else {
                        format!("Command failed: {}", result.stderr)
                    };
                    app.ui.set_status(status_msg);
                }
                Err(e) => {
                    app.ui.set_status(format!("Error executing command: {e}"));
                }
            }
        }

        Ok(())
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

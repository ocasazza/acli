//! UI rendering for the TUI application

use crate::{app::App, screens::Screen};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
    Frame,
};

/// UI state and rendering
pub struct Ui {
    /// Current status message
    pub status_message: String,
    /// Whether we're currently loading
    pub is_loading: bool,
}

impl Ui {
    /// Create a new UI instance
    pub fn new() -> Self {
        Self {
            status_message: "Ready".to_string(),
            is_loading: false,
        }
    }

    /// Draw the entire UI
    pub fn draw(&self, f: &mut Frame, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(f.size());

        // Draw main content based on current screen
        match app.current_screen {
            Screen::TreeNavigation => self.draw_tree_navigation(f, chunks[0], app),
            Screen::CommandExecution => self.draw_command_execution(f, chunks[0], app),
            Screen::MainMenu => self.draw_main_menu(f, chunks[0]),
            Screen::CqlBuilder => self.draw_cql_builder(f, chunks[0]),
            Screen::PageBrowser => self.draw_page_browser(f, chunks[0]),
            Screen::LabelManager => self.draw_label_manager(f, chunks[0]),
            Screen::Help => self.draw_help(f, chunks[0]),
        }

        // Draw footer
        self.draw_footer(f, chunks[1], &app.current_screen, app);

        // Draw loading overlay if loading
        if self.is_loading {
            self.draw_loading_overlay(f, f.size());
        }
    }

    /// Draw the footer with status and key hints
    fn draw_footer(&self, f: &mut Frame, area: Rect, screen: &Screen, app: &crate::app::App) {
        let key_hints = match screen {
            Screen::TreeNavigation => {
                if app.is_search_mode() {
                    "Type to search | Enter: Apply filter | Esc: Exit search | ↑↓: Navigate"
                } else if app.get_filtered_tree_items().is_some() {
                    "↑↓: Navigate | /: Search | Esc: Clear filter | Enter: Select | c: Commands | q: Quit"
                } else {
                    "↑↓: Navigate | ←→: Expand/Collapse | /: Search | PgUp/PgDn: Scroll | Enter: Select | c: Commands | q: Quit"
                }
            }
            Screen::CommandExecution => {
                "↑↓: Scroll Output | Enter: Execute | Esc: Back | q: Quit"
            }
            Screen::MainMenu => {
                "1: CQL Builder | 2: Page Browser | 3: Label Manager | h: Help | q: Quit"
            }
            Screen::CqlBuilder => "Enter: Execute Query | Backspace: Back | q: Quit",
            Screen::PageBrowser => "↑↓: Navigate | Enter: Select | Backspace: Back | q: Quit",
            Screen::LabelManager => "a: Add | d: Delete | u: Update | Backspace: Back | q: Quit",
            Screen::Help => "Backspace: Back | q: Quit",
        };

        let footer_text = vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::raw(&self.status_message),
            ]),
            Line::from(vec![Span::styled(
                key_hints,
                Style::default().fg(Color::Gray),
            )]),
        ];

        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        f.render_widget(footer, area);
    }

    /// Draw the main menu
    fn draw_main_menu(&self, f: &mut Frame, area: Rect) {
        let menu_items = vec![
            ListItem::new("1. CQL Query Builder - Build and execute CQL queries"),
            ListItem::new("2. Page Browser - Browse and search Confluence pages"),
            ListItem::new("3. Label Manager - Manage page labels"),
            ListItem::new("h. Help - Show help information"),
        ];

        let menu = List::new(menu_items)
            .block(
                Block::default()
                    .title("Main Menu")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ");

        f.render_widget(menu, area);
    }

    /// Draw the CQL builder screen
    fn draw_cql_builder(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(5), // Input area
                Constraint::Min(0),    // Examples/help
            ])
            .split(area);

        // Title
        let title = Paragraph::new("CQL Query Builder")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Input area (placeholder)
        let input = Paragraph::new("Type your CQL query here...")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().title("CQL Query").borders(Borders::ALL));
        f.render_widget(input, chunks[1]);

        // Examples
        let examples = [
            "Examples:",
            "• parent = 123456",
            "• space = 'DOCS' and type = 'page'",
            "• title ~ 'tutorial' and label = 'draft'",
            "• ancestor = 789012 and lastModified >= '2023-01-01'",
        ];

        let examples_widget = Paragraph::new(examples.join("\n"))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().title("CQL Examples").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        f.render_widget(examples_widget, chunks[2]);
    }

    /// Draw the page browser screen
    fn draw_page_browser(&self, f: &mut Frame, area: Rect) {
        let title = Paragraph::new("Page Browser")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, area);
    }

    /// Draw the label manager screen
    fn draw_label_manager(&self, f: &mut Frame, area: Rect) {
        let title = Paragraph::new("Label Manager")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, area);
    }

    /// Draw the help screen
    fn draw_help(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            "ACLI TUI Help",
            "",
            "Navigation:",
            "• Use number keys in main menu to select options",
            "• Arrow keys to navigate lists",
            "• Enter to select/confirm",
            "• Backspace to go back",
            "• q or Esc to quit",
            "",
            "Screens:",
            "• Main Menu: Select different operations",
            "• CQL Builder: Create Confluence Query Language expressions",
            "• Page Browser: View and navigate page results",
            "• Label Manager: Add, update, or remove page labels",
            "",
            "Environment Variables Required:",
            "• ATLASSIAN_URL: Your Atlassian instance URL",
            "• ATLASSIAN_USERNAME: Your username/email",
            "• ATLASSIAN_API_TOKEN: Your API token",
        ];

        let help = Paragraph::new(help_text.join("\n"))
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(help, area);
    }

    /// Draw loading overlay
    fn draw_loading_overlay(&self, f: &mut Frame, area: Rect) {
        let loading_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(3),
                Constraint::Percentage(40),
            ])
            .split(area)[1];

        let loading_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(loading_area)[1];

        f.render_widget(Clear, loading_area);

        let loading = Paragraph::new("Loading...")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Yellow)),
            );
        f.render_widget(loading, loading_area);
    }

    /// Set the status message
    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
    }

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
        if loading {
            self.status_message = "Loading...".to_string();
        }
    }

    /// Draw the tree navigation screen
    fn draw_tree_navigation(&self, f: &mut Frame, area: Rect, app: &App) {
        let main_chunks = if app.is_search_mode() {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Search input
                    Constraint::Min(0),    // Tree and context
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)])
                .split(area)
        };

        // Draw search input if in search mode
        if app.is_search_mode() {
            let search_text = format!("Search: {}", app.get_search_query());
            let search_input = Paragraph::new(search_text)
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .block(
                    Block::default()
                        .title("🔍 Search Mode")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Yellow)),
                );
            f.render_widget(search_input, main_chunks[0]);
        }

        let tree_area = if app.is_search_mode() {
            main_chunks[1]
        } else {
            main_chunks[0]
        };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Tree view area (will contain tree + scrollbar)
                Constraint::Percentage(40), // Context panel
            ])
            .split(tree_area);

        // Split the tree area to make room for scrollbar
        let tree_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),    // Tree list
                Constraint::Length(1), // Scrollbar
            ])
            .split(chunks[0]);

        // Get display items (filtered or full tree)
        let tree_items_data = app.get_display_items();

        // Create tree items for display with fuzzy highlighting
        let tree_items: Vec<ListItem> = if let Some(fuzzy_items) = app.get_fuzzy_display_items() {
            // Use fuzzy search results with highlighting
            fuzzy_items
                .iter()
                .map(
                    |(name, _depth, selected, _score, match_positions, _original_index)| {
                        // Create highlighted text spans
                        let highlighted_spans = self.create_highlighted_spans(
                            name,
                            match_positions,
                            app.get_search_query(),
                        );

                        let base_style = if *selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        };

                        ListItem::new(Line::from(highlighted_spans)).style(base_style)
                    },
                )
                .collect()
        } else {
            // Use regular tree items without highlighting
            tree_items_data
                .iter()
                .map(|(name, _depth, selected)| {
                    let style = if *selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(name.clone()).style(style)
                })
                .collect()
        };

        // Build tree title
        let tree_title = if app.get_filtered_tree_items().is_some() {
            format!("🔍 Filtered Results ({} items)", tree_items_data.len())
        } else {
            app.domain
                .as_ref()
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "Atlassian Navigation".to_string())
        };

        let tree = List::new(tree_items)
            .block(
                Block::default()
                    .title(tree_title)
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol("▶ ")
            .start_corner(ratatui::layout::Corner::TopLeft);

        // Create list state and render with proper scrolling
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(app.get_tree_selection()));

        f.render_stateful_widget(tree, tree_chunks[0], &mut list_state);

        // Calculate scrollbar parameters
        let content_length = tree_items_data.len();
        let viewport_height = tree_chunks[0].height.saturating_sub(2) as usize; // Account for borders

        // Create and render scrollbar if needed
        if content_length > viewport_height {
            // Calculate scroll position from list state
            let scroll_position = if app.get_tree_selection() >= viewport_height {
                app.get_tree_selection().saturating_sub(viewport_height - 1)
            } else {
                0
            };

            let max_scroll = content_length.saturating_sub(viewport_height);
            let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll_position);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(Color::Gray))
                .thumb_style(Style::default().fg(Color::White));

            f.render_stateful_widget(scrollbar, tree_chunks[1], &mut scrollbar_state);
        }

        // Context panel
        let display_path = app.get_navigation_context().display_path();
        let mut context_lines = vec![
            "Current Selection:".to_string(),
            "".to_string(),
            display_path,
            "".to_string(),
        ];

        // Add search info if filtering
        if let Some(filtered) = app.get_filtered_tree_items() {
            context_lines.push(format!("🔍 Search: '{}'", app.get_search_query()));
            context_lines.push(format!("Found {} matches", filtered.len()));
            context_lines.push("".to_string());
        }

        context_lines.extend(vec![
            "Actions:".to_string(),
            "• Press Enter to select/expand".to_string(),
            "• Press 'c' for commands when project selected".to_string(),
            "• Press '/' to search".to_string(),
            "• Use arrow keys to navigate".to_string(),
            "".to_string(),
            if app.get_navigation_context().is_complete() {
                "✅ Complete context - Commands available".to_string()
            } else {
                "⚠️  Select a project to enable commands".to_string()
            },
        ]);

        let context_panel = Paragraph::new(context_lines.join("\n"))
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title("Context")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(context_panel, chunks[1]);
    }

    /// Draw the command execution screen
    fn draw_command_execution(&self, f: &mut Frame, area: Rect, app: &App) {
        use crate::command::{AvailableCommand, CommandInputMode};

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Context header
                Constraint::Min(6),    // Command selection and input
                Constraint::Length(8), // Results
            ])
            .split(area);

        // Context header
        let context_text = format!("Context: {}", app.get_navigation_context().display_path());
        let header = Paragraph::new(context_text)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        // Command selection and input area
        let cmd_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Available commands
                Constraint::Percentage(50), // Command input/details
            ])
            .split(chunks[1]);

        // Available commands list
        let available_commands = app.get_available_commands();
        let command_items: Vec<ListItem> = available_commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let (name, description) = match cmd {
                    AvailableCommand::Ctag {
                        operation,
                        description,
                    } => (operation.as_str(), description.as_str()),
                };

                let style = if i == app.command_selection {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let text = format!("{name} - {description}");
                ListItem::new(text).style(style)
            })
            .collect();

        let commands_list = List::new(command_items)
            .block(
                Block::default()
                    .title("Available Commands")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("▶ ");

        f.render_widget(commands_list, cmd_chunks[0]);

        // Command input/details area
        match app.command_input.mode {
            CommandInputMode::SelectingCommand => {
                let help_text = [
                    "Select a command:",
                    "",
                    "• Use ↑↓ to navigate",
                    "• Press Enter to select",
                    "• Press first letter for quick selection",
                    "• Press Esc to go back",
                ];

                let help_widget = Paragraph::new(help_text.join("\n"))
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().title("Instructions").borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(help_widget, cmd_chunks[1]);
            }
            CommandInputMode::TypingArgs => {
                let cql_context = app
                    .get_navigation_context()
                    .cql_context()
                    .unwrap_or_else(|| "No context available".to_string());

                let selected_cmd = if let Some(AvailableCommand::Ctag { operation, .. }) =
                    &app.command_input.selected_command
                {
                    operation.as_str()
                } else {
                    "unknown"
                };

                let command_preview = format!(
                    "ctag {} \"{}\" {}",
                    selected_cmd, cql_context, app.command_input.text
                );

                let input_text = [
                    format!("Command: {selected_cmd}"),
                    format!("CQL Context: {cql_context}"),
                    format!("Additional Args: {}", app.command_input.text),
                    "".to_string(),
                    "Full Command:".to_string(),
                    command_preview,
                ];

                let input_widget = Paragraph::new(input_text.join("\n"))
                    .style(Style::default().fg(Color::Green))
                    .block(
                        Block::default()
                            .title("Command Builder")
                            .borders(Borders::ALL),
                    )
                    .wrap(Wrap { trim: true });
                f.render_widget(input_widget, cmd_chunks[1]);
            }
            CommandInputMode::Ready => {
                let ready_text = [
                    "Command ready to execute!",
                    "",
                    "Press Enter to run the command",
                    "Press Esc to go back",
                ];

                let ready_widget = Paragraph::new(ready_text.join("\n"))
                    .style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                    .block(Block::default().title("Ready").borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(ready_widget, cmd_chunks[1]);
            }
        }

        // Results area
        if !app.command_output.is_empty() {
            let result_text: Vec<Line> = app
                .command_output
                .iter()
                .map(|line| Line::from(line.clone()))
                .collect();

            let results_widget = Paragraph::new(result_text)
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .title("Command Output (scroll ↑↓)")
                        .borders(Borders::ALL),
                )
                .scroll((app.command_output_scroll as u16, 0));
            f.render_widget(results_widget, chunks[2]);
        } else {
            let placeholder_widget = Paragraph::new(
                "No command output. Select a command and press Enter to execute.",
            )
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .title("Command Output")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true });
            f.render_widget(placeholder_widget, chunks[2]);
        }
    }

    /// Create highlighted text spans for fuzzy search matches
    fn create_highlighted_spans(
        &self,
        text: &str,
        match_positions: &[usize],
        _query: &str,
    ) -> Vec<Span> {
        if match_positions.is_empty() {
            return vec![Span::raw(text.to_string())];
        }

        let mut spans = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut last_pos = 0;

        // Sort positions to ensure we process them in order
        let mut sorted_positions = match_positions.to_vec();
        sorted_positions.sort_unstable();
        sorted_positions.dedup();

        for &pos in &sorted_positions {
            if pos < chars.len() {
                // Add non-highlighted text before this match
                if pos > last_pos {
                    let segment: String = chars[last_pos..pos].iter().collect();
                    if !segment.is_empty() {
                        spans.push(Span::styled(segment, Style::default().fg(Color::White)));
                    }
                }

                // Add highlighted character with bright color and bold
                spans.push(Span::styled(
                    chars[pos].to_string(),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ));
                last_pos = pos + 1;
            }
        }

        // Add remaining non-highlighted text
        if last_pos < chars.len() {
            let segment: String = chars[last_pos..].iter().collect();
            if !segment.is_empty() {
                spans.push(Span::styled(segment, Style::default().fg(Color::White)));
            }
        }

        spans
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

//! UI rendering for the TUI application

use crate::{app::App, screens::Screen};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
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
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(f.size());

        // Draw header
        self.draw_header(f, chunks[0]);

        // Draw main content based on current screen
        match app.current_screen {
            Screen::MainMenu => self.draw_main_menu(f, chunks[1]),
            Screen::CqlBuilder => self.draw_cql_builder(f, chunks[1]),
            Screen::PageBrowser => self.draw_page_browser(f, chunks[1]),
            Screen::LabelManager => self.draw_label_manager(f, chunks[1]),
            Screen::Help => self.draw_help(f, chunks[1]),
        }

        // Draw footer
        self.draw_footer(f, chunks[2], &app.current_screen);

        // Draw loading overlay if loading
        if self.is_loading {
            self.draw_loading_overlay(f, f.size());
        }
    }

    /// Draw the header
    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let header = Paragraph::new("ACLI - Atlassian Command Line Interface")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, area);
    }

    /// Draw the footer with status and key hints
    fn draw_footer(&self, f: &mut Frame, area: Rect, screen: &Screen) {
        let key_hints = match screen {
            Screen::MainMenu => "1: CQL Builder | 2: Page Browser | 3: Label Manager | h: Help | q: Quit",
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
            Line::from(vec![Span::styled(key_hints, Style::default().fg(Color::Gray))]),
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
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Input area (placeholder)
        let input = Paragraph::new("Type your CQL query here...")
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .title("CQL Query")
                    .borders(Borders::ALL),
            );
        f.render_widget(input, chunks[1]);

        // Examples
        let examples = vec![
            "Examples:",
            "• parent = 123456",
            "• space = 'DOCS' and type = 'page'",
            "• title ~ 'tutorial' and label = 'draft'",
            "• ancestor = 789012 and lastModified >= '2023-01-01'",
        ];

        let examples_widget = Paragraph::new(examples.join("\n"))
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .title("CQL Examples")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(examples_widget, chunks[2]);
    }

    /// Draw the page browser screen
    fn draw_page_browser(&self, f: &mut Frame, area: Rect) {
        let title = Paragraph::new("Page Browser")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, area);
    }

    /// Draw the label manager screen
    fn draw_label_manager(&self, f: &mut Frame, area: Rect) {
        let title = Paragraph::new("Label Manager")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
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
            "• ATLASSIAN_TOKEN: Your API token",
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
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
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
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

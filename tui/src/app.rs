//! Main TUI application state and event handling

use crate::{
    command::{AvailableCommand, CommandExecutor, CommandInput},
    create_confluence_client,
    domain_loader::DomainLoader,
    event_handler::EventHandler,
    models::{AtlassianDomain, NavigationContext, TreeItem, TreeItemWithMetadata},
    screens::Screen,
    search::SearchManager,
    terminal_manager::TerminalManager,
    tree_navigation::TreeNavigationManager,
    ui::Ui,
};
use crossterm::event::{self, Event};
use nix_rust_template::ConfluenceClient;
use ratatui::{backend::Backend, Terminal};
use std::{error::Error, time::Duration};

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
    /// Tree navigation manager
    pub tree_navigation: TreeNavigationManager,
    /// Search manager
    pub search_manager: SearchManager,
    /// Available domain loaded from environment
    pub domain: Option<AtlassianDomain>,
    /// Command executor for running CLI commands
    pub command_executor: CommandExecutor,
    /// Command input state
    pub command_input: CommandInput,
    /// Current command selection index (for selecting from available commands)
    pub command_selection: usize,
    /// Command output
    pub command_output: Vec<String>,
    /// Command output scroll position
    pub command_output_scroll: usize,
}

impl App {
    /// Create a new App instance
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let confluence_client = create_confluence_client()?;
        let ui = Ui::new();

        let tree_navigation = TreeNavigationManager::new();
        let command_executor = CommandExecutor::new(tree_navigation.navigation_context.clone());
        let search_manager = SearchManager::new();

        let mut app = Self {
            should_quit: false,
            current_screen: Screen::TreeNavigation,
            confluence_client,
            ui,
            tree_navigation,
            search_manager,
            domain: None,
            command_executor,
            command_input: CommandInput::new(),
            command_selection: 0,
            command_output: Vec::new(),
            command_output_scroll: 0,
        };

        // Load domain data from environment
        let confluence_client_copy = create_confluence_client()?;
        app.load_domain_data(confluence_client_copy)?;

        Ok(app)
    }

    /// Run the TUI application
    pub fn run(mut self) -> Result<(), Box<dyn Error>> {
        // Setup terminal
        let mut terminal = TerminalManager::setup()?;

        // Main application loop
        let result = self.run_app(&mut terminal);

        // Enhanced cleanup
        self.cleanup_resources();

        // Cleanup terminal
        TerminalManager::cleanup(&mut terminal)?;

        result
    }

    /// Main application event loop
    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
        loop {
            // Draw UI
            terminal.draw(|f| self.ui.draw(f, self))?;

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

    /// Clean up resources to prevent memory leaks
    fn cleanup_resources(&mut self) {
        self.tree_navigation.cleanup();
        self.search_manager.cleanup();
        self.domain = None;
    }

    /// Handle incoming events
    fn handle_event(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
        EventHandler::handle_event(self, event)
    }

    /// Switch to a different screen
    pub fn switch_screen(&mut self, screen: Screen) {
        self.current_screen = screen;
    }

    /// Load domain data from environment variables and discover products/projects
    fn load_domain_data(
        &mut self,
        confluence_client: ConfluenceClient,
    ) -> Result<(), Box<dyn Error>> {
        let domain_loader = DomainLoader::new(confluence_client);
        let domain = domain_loader.load_domain_data()?;

        self.tree_navigation.build_tree_data(domain.clone());
        self.domain = Some(domain);

        Ok(())
    }

    /// Get available commands for the current context
    pub fn get_available_commands(&self) -> Vec<AvailableCommand> {
        self.command_executor.get_available_commands()
    }

    /// Get the most recent command result
    pub fn get_last_command_result(&self) -> Option<&crate::command::CommandResult> {
        self.command_executor.get_last_result()
    }

    /// Get all visible tree items for display (flattened with indentation)
    pub fn get_tree_items(&self) -> Vec<TreeItem> {
        self.tree_navigation.get_tree_items()
    }

    /// Get the items to display (either filtered or full tree)
    pub fn get_display_items(&self) -> Vec<TreeItem> {
        let tree_items = self.tree_navigation.get_tree_items();
        self.search_manager.get_display_items(&tree_items)
    }

    /// Get fuzzy search results with highlighting information
    pub fn get_fuzzy_display_items(&self) -> Option<&Vec<TreeItemWithMetadata>> {
        self.search_manager.get_fuzzy_display_items()
    }

    /// Get current tree selection index
    pub fn get_tree_selection(&self) -> usize {
        self.tree_navigation.tree_selection
    }

    /// Check if in search mode
    pub fn is_search_mode(&self) -> bool {
        self.search_manager.search_mode
    }

    /// Get current search query
    pub fn get_search_query(&self) -> &str {
        &self.search_manager.search_query
    }

    /// Get navigation context
    pub fn get_navigation_context(&self) -> &NavigationContext {
        &self.tree_navigation.navigation_context
    }

    /// Get filtered tree items
    pub fn get_filtered_tree_items(&self) -> Option<&Vec<TreeItemWithMetadata>> {
        self.search_manager.filtered_tree_items.as_ref()
    }
}

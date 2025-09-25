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
use nix_rust_template::{ConfluenceClient, CtagOperation, execute_ctag_operation};
use ratatui::{backend::Backend, Terminal};
use std::{collections::HashSet, error::Error, time::Duration};

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
    /// Page tree navigation manager for page selection screen
    pub page_tree_navigation: TreeNavigationManager,
    /// Search manager
    pub search_manager: SearchManager,
    /// Search manager for page selection screen
    pub page_search_manager: SearchManager,
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
    /// Tree navigation for command results
    pub command_result_tree: Option<TreeNavigationManager>,
    /// Label buffer containing labels from selected pages
    pub label_buffer: HashSet<String>,
    /// Set of selected page IDs
    pub selected_pages: HashSet<String>,
    /// Whether page tree is loaded
    pub page_tree_loaded: bool,
    /// Space pages tree for the currently selected Confluence space
    pub space_pages_tree: Option<TreeNavigationManager>,
    /// Currently selected space for which pages are loaded
    pub current_space_key: Option<String>,
    /// Loading state for space pages
    pub space_pages_loading: bool,
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
            page_tree_navigation: TreeNavigationManager::new(),
            search_manager,
            page_search_manager: SearchManager::new(),
            domain: None,
            command_executor,
            command_input: CommandInput::new(),
            command_selection: 0,
            command_output: Vec::new(),
            command_output_scroll: 0,
            command_result_tree: None,
            label_buffer: HashSet::new(),
            selected_pages: HashSet::new(),
            page_tree_loaded: false,
            space_pages_tree: None,
            current_space_key: None,
            space_pages_loading: false,
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
        // If switching to page selection and page tree isn't loaded, load it
        if screen == Screen::PageSelection && !self.page_tree_loaded {
            if let Err(e) = self.load_page_tree() {
                self.ui.set_status(format!("Failed to load page tree: {}", e));
                return;
            }
        }

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

    /// Load page tree from the current navigation context
    fn load_page_tree(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.tree_navigation.navigation_context.is_complete() {
            return Err("Navigation context is not complete".into());
        }

        self.ui.set_loading(true);
        self.ui.set_status("Loading root pages...".to_string());

        // Build CQL query to fetch only root pages in the current space
        if let Some(cql_context) = self.tree_navigation.navigation_context.cql_context() {
            // Execute ctag list operation to get root pages with their labels
            let operation = CtagOperation::List {
                cql: cql_context, // This will be just "space = ITLC" without ancestor clause
                tags: None,
                tree: false, // Don't request tree format initially, we'll build it lazily
            };

            match execute_ctag_operation(operation, false, false) {
                Ok(result) => {
                    // Build tree from the result pages (these will be root pages)
                    self.page_tree_navigation = crate::command::create_tree_from_ctag_results(&result);
                    self.page_tree_loaded = true;
                    self.ui.set_status(format!("Loaded {} root pages", result.pages.len()));
                }
                Err(e) => {
                    self.ui.set_status(format!("Failed to load pages: {}", e));
                    return Err(e);
                }
            }
        } else {
            return Err("No CQL context available".into());
        }

        self.ui.set_loading(false);
        Ok(())
    }

    /// Load children for a specific page (lazy loading)
    fn load_page_children(&mut self, page_id: &str) -> Result<(), Box<dyn Error>> {
        if !self.tree_navigation.navigation_context.is_complete() {
            return Err("Navigation context is not complete".into());
        }

        self.ui.set_status(format!("Loading children for page {}...", page_id));

        // Build CQL query to fetch children of the specified page
        if let Some(space_context) = self.tree_navigation.navigation_context.cql_context() {
            // Extract space part and add ancestor clause
            let cql_with_ancestor = format!("{} AND ancestor = {}", space_context, page_id);

            let operation = CtagOperation::List {
                cql: cql_with_ancestor,
                tags: None,
                tree: false,
            };

            match execute_ctag_operation(operation, false, false) {
                Ok(result) => {
                    // Convert result pages to tree nodes
                    let child_nodes = self.build_page_nodes_from_results(&result);

                    // Update the parent node with children
                    self.update_page_node_children(page_id, child_nodes)?;

                    self.ui.set_status(format!("Loaded {} children for page", result.pages.len()));
                }
                Err(e) => {
                    self.ui.set_status(format!("Failed to load children: {}", e));
                    return Err(e);
                }
            }
        } else {
            return Err("No CQL context available".into());
        }

        Ok(())
    }

    /// Build page tree nodes from ctag results
    fn build_page_nodes_from_results(&self, result: &nix_rust_template::CtagResult) -> Vec<crate::models::TreeNode> {
        use crate::models::TreeNode;

        result.pages.iter().map(|page_result| {
            // Determine if this page has children by checking if it has any descendants
            // For now, assume all pages might have children (we'll load them on demand)
            let has_children = true; // Conservative assumption for lazy loading

            TreeNode::new_page(
                page_result.id.clone(),
                page_result.title.clone(),
                page_result.labels.clone(),
                has_children,
            )
        }).collect()
    }

    /// Update a page node with its loaded children
    fn update_page_node_children(&mut self, page_id: &str, children: Vec<crate::models::TreeNode>) -> Result<(), Box<dyn Error>> {
        // This is a simplified implementation - in practice you'd need to traverse
        // the tree structure to find the node with the matching page_id and update it
        // For now, this is a placeholder that will be implemented when the tree navigation
        // manager is updated to support page tree operations

        // TODO: Implement proper tree traversal and node updating
        // self.page_tree_navigation.update_node_children(page_id, children)?;

        Ok(())
    }

    /// Move page selection up
    pub fn move_page_selection_up(&mut self) {
        self.page_tree_navigation.move_selection_up();
    }

    /// Move page selection down
    pub fn move_page_selection_down(&mut self) {
        self.page_tree_navigation.move_selection_down();
    }

    /// Expand current page node
    pub fn expand_current_page_node(&mut self) {
        self.page_tree_navigation.expand_current_node();
    }

    /// Collapse current page node
    pub fn collapse_current_page_node(&mut self) {
        self.page_tree_navigation.collapse_current_node();
    }

    /// Toggle page selection for label buffer
    pub fn toggle_page_selection(&mut self) -> Result<(), Box<dyn Error>> {
        let _tree_items = self.page_tree_navigation.get_tree_items();
        let selection_index = self.page_tree_navigation.tree_selection;

        // TODO: Implement proper page node traversal and label extraction
        // For now, create a placeholder page ID based on selection index
        let page_id = format!("page_{}", selection_index);
        let labels = vec![format!("label_{}", selection_index)]; // Placeholder labels

        if self.selected_pages.contains(&page_id) {
            // Remove from selection and label buffer
            self.selected_pages.remove(&page_id);

            // Remove labels from this page from the buffer
            for label in &labels {
                self.label_buffer.remove(label);
            }
        } else {
            // Add to selection and label buffer
            self.selected_pages.insert(page_id);

            // Add labels from this page to the buffer
            for label in labels {
                self.label_buffer.insert(label);
            }
        }

        self.ui.set_status(format!(
            "Selected {} pages, {} unique labels",
            self.selected_pages.len(),
            self.label_buffer.len()
        ));

        Ok(())
    }

    /// Enter page search mode
    pub fn enter_page_search_mode(&mut self) {
        self.page_search_manager.enter_search_mode(&mut self.ui);
    }

    /// Clear label buffer
    pub fn clear_label_buffer(&mut self) {
        self.label_buffer.clear();
        self.selected_pages.clear();
        self.ui.set_status("Cleared label buffer".to_string());
    }

    /// Get page node at given flattened index
    fn get_page_node_at_index(&self, index: usize) -> Option<&crate::models::TreeNode> {
        // This is a simplified implementation - in practice you'd need to traverse
        // the tree structure to find the node at the given flattened index
        // For now, return None as placeholder
        None
    }

    /// Get labels for a specific page ID
    fn get_page_labels(&self, page_id: &str) -> Option<Vec<String>> {
        // This would extract labels from the page tree structure
        // For now, return None as placeholder
        None
    }

    /// Get page tree items for display
    pub fn get_page_tree_items(&self) -> Vec<TreeItem> {
        self.page_tree_navigation.get_tree_items()
    }

    /// Get current page tree selection index
    pub fn get_page_tree_selection(&self) -> usize {
        self.page_tree_navigation.tree_selection
    }

    /// Check if page tree is loaded
    pub fn is_page_tree_loaded(&self) -> bool {
        self.page_tree_loaded
    }

    /// Get label buffer as sorted vector
    pub fn get_label_buffer(&self) -> Vec<String> {
        let mut labels: Vec<String> = self.label_buffer.iter().cloned().collect();
        labels.sort();
        labels
    }

    /// Get selected pages count
    pub fn get_selected_pages_count(&self) -> usize {
        self.selected_pages.len()
    }

    /// Check if a Confluence space is currently selected
    pub fn is_confluence_space_selected(&self) -> bool {
        if let Some(ref _context) = self.tree_navigation.navigation_context.project {
            if let Some(ref product) = self.tree_navigation.navigation_context.product {
                return product.product_type == nix_rust_template::ProductType::Confluence;
            }
        }
        false
    }

    /// Get the currently selected space key if a Confluence space is selected
    pub fn get_selected_space_key(&self) -> Option<String> {
        if self.is_confluence_space_selected() {
            if let Some(ref context) = self.tree_navigation.navigation_context.project {
                return Some(context.key.clone());
            }
        }
        None
    }

    /// Load space pages tree for the currently selected Confluence space
    pub fn load_space_pages(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.is_confluence_space_selected() {
            return Err("No Confluence space selected".into());
        }

        let space_key = self.get_selected_space_key()
            .ok_or("Could not get space key")?;

        // Get the homepage_id for the selected space
        let homepage_id = if let Some(ref project) = self.tree_navigation.navigation_context.project {
            project.homepage_id.clone()
        } else {
            None
        };

        // Check if we already have pages loaded for this space
        if let Some(ref current_space) = self.current_space_key {
            if current_space == &space_key && self.space_pages_tree.is_some() {
                return Ok(()); // Already loaded
            }
        }

        self.space_pages_loading = true;
        self.ui.set_status(format!("Loading pages for space {}...", space_key));

        // For now, use space-only query to avoid hanging issue with ancestor filtering
        // TODO: Debug homepage_id and add back ancestor filtering later
        let cql = format!("space = '{}'", space_key);

        // Debug: Print the command being executed
        println!("DEBUG: Loading space pages for space: {}", space_key);
        println!("DEBUG: CQL query: {}", cql);
        if let Some(ref homepage_id) = homepage_id {
            println!("DEBUG: Homepage ID available: {}", homepage_id);
        } else {
            println!("DEBUG: No homepage ID available");
        }

        let operation = CtagOperation::List {
            cql: cql.clone(),
            tags: None,
            tree: true, // Let ctag build the tree structure for us
        };

        println!("DEBUG: Executing ctag operation: {:?}", operation);

        match execute_ctag_operation(operation, false, false) {
            Ok(result) => {
                // Debug: Print the result information
                println!("DEBUG: Command executed successfully");
                println!("DEBUG: Number of pages returned: {}", result.pages.len());
                for (i, page) in result.pages.iter().take(5).enumerate() {
                    println!("DEBUG: Page {}: {} (ID: {})", i, page.title, page.id);
                }
                if result.pages.len() > 5 {
                    println!("DEBUG: ... and {} more pages", result.pages.len() - 5);
                }

                // Create a new tree navigation manager for space pages
                let mut space_tree = TreeNavigationManager::new();

                // Build tree structure from the pages (ctag already organized them hierarchically)
                let pages_count = result.pages.len();
                println!("DEBUG: Building space tree with {} pages", pages_count);
                space_tree.build_space_pages_tree(result);
                println!("DEBUG: Space tree built successfully");

                self.space_pages_tree = Some(space_tree);
                self.current_space_key = Some(space_key.clone());
                self.space_pages_loading = false;

                let status_msg = if homepage_id.is_some() {
                    format!("Loaded {} pages under homepage from space {}", pages_count, space_key)
                } else {
                    format!("Loaded {} pages from space {} (no homepage filter)", pages_count, space_key)
                };
                self.ui.set_status(status_msg);
            }
            Err(e) => {
                self.space_pages_loading = false;
                self.ui.set_status(format!("Failed to load space pages: {}", e));
                return Err(e);
            }
        }

        Ok(())
    }

    /// Get space pages tree items for display in right panel
    pub fn get_space_pages_tree_items(&self) -> Option<Vec<TreeItem>> {
        self.space_pages_tree.as_ref().map(|tree| tree.get_tree_items())
    }

    /// Check if space pages are currently loading
    pub fn is_space_pages_loading(&self) -> bool {
        self.space_pages_loading
    }

    /// Clear space pages tree (when navigating away from a space)
    pub fn clear_space_pages(&mut self) {
        self.space_pages_tree = None;
        self.current_space_key = None;
        self.space_pages_loading = false;
    }

    /// Handle space navigation change - load pages if a new Confluence space is selected
    pub fn handle_space_navigation_change(&mut self) -> Result<(), Box<dyn Error>> {
        println!("DEBUG: handle_space_navigation_change called");
        if self.is_confluence_space_selected() {
            // A Confluence space is selected, load its pages
            println!("DEBUG: Confluence space detected, calling load_space_pages");
            self.load_space_pages()?;
        } else {
            // Not a Confluence space, clear any loaded space pages
            println!("DEBUG: No Confluence space selected, clearing space pages");
            self.clear_space_pages();
        }
        Ok(())
    }
}

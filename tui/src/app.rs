//! Main TUI application state and event handling

use crate::{create_confluence_client, screens::Screen, ui::Ui, models::{NavigationContext, AtlassianDomain, AtlassianProduct, ProductType, Project, TreeNode}, command::{CommandExecutor, CommandInput, AvailableCommand, TuiCommand}};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind},
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
    /// Navigation context for hierarchical selection
    pub navigation_context: NavigationContext,
    /// Tree data for navigation
    pub tree_data: Vec<TreeNode>,
    /// Current tree selection index
    pub tree_selection: usize,
    /// Available domain loaded from environment
    pub domain: Option<AtlassianDomain>,
    /// Command executor for running CLI commands
    pub command_executor: CommandExecutor,
    /// Command input state
    pub command_input: CommandInput,
    /// Current command selection index (for selecting from available commands)
    pub command_selection: usize,
    /// Search mode state
    pub search_mode: bool,
    /// Current search query
    pub search_query: String,
    /// Filtered tree items (when searching)
    pub filtered_tree_items: Option<Vec<(String, usize, bool)>>,
}

impl App {
    /// Create a new App instance
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let confluence_client = create_confluence_client()?;
        let ui = Ui::new();

        let navigation_context = NavigationContext::new();
        let command_executor = CommandExecutor::new(navigation_context.clone());

        let mut app = Self {
            should_quit: false,
            current_screen: Screen::TreeNavigation,
            confluence_client,
            ui,
            navigation_context,
            tree_data: Vec::new(),
            tree_selection: 0,
            domain: None,
            command_executor,
            command_input: CommandInput::new(),
            command_selection: 0,
            search_mode: false,
            search_query: String::new(),
            filtered_tree_items: None,
        };

        // Load domain data from environment
        app.load_domain_data()?;

        Ok(app)
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
                            Screen::TreeNavigation => {
                                if self.search_mode {
                                    self.handle_search_input(code)?;
                                } else {
                                    match code {
                                        KeyCode::Enter => {
                                            self.select_current_node();
                                        }
                                        KeyCode::Up => {
                                            self.move_tree_selection_up();
                                        }
                                        KeyCode::Down => {
                                            self.move_tree_selection_down();
                                        }
                                        KeyCode::Right => {
                                            self.expand_current_node();
                                        }
                                        KeyCode::Left => {
                                            self.collapse_current_node();
                                        }
                                        KeyCode::Char('c') => {
                                            // Switch to command execution for ctag
                                            if self.navigation_context.is_complete() {
                                                self.switch_screen(Screen::CommandExecution);
                                            }
                                        }
                                        KeyCode::Char('/') => {
                                            // Enter search mode
                                            self.enter_search_mode();
                                        }
                                        KeyCode::PageUp => {
                                            // Page up - move selection up by ~10 items
                                            for _ in 0..10 {
                                                if self.tree_selection > 0 {
                                                    self.tree_selection -= 1;
                                                } else {
                                                    break;
                                                }
                                            }
                                        }
                                        KeyCode::PageDown => {
                                            // Page down - move selection down by ~10 items
                                            let tree_items = self.get_display_items();
                                            for _ in 0..10 {
                                                if self.tree_selection < tree_items.len().saturating_sub(1) {
                                                    self.tree_selection += 1;
                                                } else {
                                                    break;
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Screen::CommandExecution => {
                                self.handle_command_execution_input(code)?;
                            }
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
            Event::Mouse(MouseEvent { kind, .. }) => {
                // Handle mouse events (scroll wheel)
                match kind {
                    MouseEventKind::ScrollUp => {
                        if self.current_screen == Screen::TreeNavigation && !self.search_mode {
                            self.move_tree_selection_up();
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        if self.current_screen == Screen::TreeNavigation && !self.search_mode {
                            self.move_tree_selection_down();
                        }
                    }
                    _ => {}
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

    /// Load domain data from environment variables and discover products/projects
    fn load_domain_data(&mut self) -> Result<(), Box<dyn Error>> {
        dotenv::dotenv().ok(); // Load .env file, ignore if not found

        let base_url = std::env::var("ATLASSIAN_URL")
            .map_err(|_| "ATLASSIAN_URL environment variable not set")?;
        let _username = std::env::var("ATLASSIAN_USERNAME")
            .map_err(|_| "ATLASSIAN_USERNAME environment variable not set")?;

        // Extract domain name from URL
        let domain_name = if let Ok(url) = url::Url::parse(&base_url) {
            url.host_str().unwrap_or(&base_url).to_string()
        } else {
            base_url.clone()
        };

        // Create domain
        let mut domain = AtlassianDomain {
            name: domain_name,
            base_url: base_url.clone(),
            products: Vec::new(),
        };

        // Try to discover Confluence and load spaces
        let confluence_product = match self.confluence_client.get_spaces() {
            Ok(spaces) => {
                let confluence_projects: Vec<Project> = spaces
                    .into_iter()
                    .map(|space| Project {
                        id: space.id,
                        name: space.name,
                        key: space.key,
                        description: space.description
                            .and_then(|d| d.plain)
                            .map(|p| p.value),
                        project_type: "space".to_string(),
                    })
                    .collect();

                AtlassianProduct {
                    product_type: ProductType::Confluence,
                    name: "Confluence".to_string(),
                    projects: confluence_projects,
                    available: true,
                }
            }
            Err(e) => {
                // Log the actual error for debugging
                eprintln!("Confluence API error: {e:?}");
                AtlassianProduct {
                    product_type: ProductType::Confluence,
                    name: format!("Confluence (Error: {e})"),
                    projects: Vec::new(),
                    available: false,
                }
            }
        };

        domain.products.push(confluence_product);

        // Add placeholder for other products
        domain.products.push(AtlassianProduct {
            product_type: ProductType::Jira,
            name: "Jira (coming soon)".to_string(),
            projects: Vec::new(),
            available: false,
        });

        domain.products.push(AtlassianProduct {
            product_type: ProductType::Jsm,
            name: "Jira Service Management (coming soon)".to_string(),
            projects: Vec::new(),
            available: false,
        });

        self.domain = Some(domain.clone());
        self.build_tree_data(domain);

        Ok(())
    }

    /// Build tree data structure from domain
    fn build_tree_data(&mut self, domain: AtlassianDomain) {
        let mut tree_nodes = Vec::new();

        for product in &domain.products {
            let mut product_node = TreeNode::new_product(product.clone());

            // Expand Confluence by default if it has projects
            if product.product_type == ProductType::Confluence && !product.projects.is_empty() {
                product_node.expanded = true;
            }

            for project in &product.projects {
                let project_node = TreeNode::new_project(project.clone());
                product_node.children.push(project_node);
            }

            tree_nodes.push(product_node);
        }

        self.tree_data = tree_nodes;
    }

    /// Get all visible tree items for display (flattened with indentation)
    pub fn get_tree_items(&self) -> Vec<(String, usize, bool)> {
        let mut items = Vec::new();
        for node in &self.tree_data {
            self.flatten_tree_node(node, 0, &mut items);
        }
        items
    }

    /// Recursively flatten tree nodes for display
    fn flatten_tree_node(&self, node: &TreeNode, depth: usize, items: &mut Vec<(String, usize, bool)>) {
        let prefix = "  ".repeat(depth);
        let icon = match &node.node_type {
            crate::models::TreeNodeType::Domain(_) => "🌐",
            crate::models::TreeNodeType::Product(product) => {
                if product.available {
                    match product.product_type {
                        ProductType::Confluence => "📊",
                        ProductType::Jira => "📋",
                        ProductType::Jsm => "🎫",
                    }
                } else {
                    "⭕"
                }
            },
            crate::models::TreeNodeType::Project(_) => "📁",
        };

        let expand_icon = if !node.children.is_empty() {
            if node.expanded { "▼ " } else { "▶ " }
        } else {
            "  "
        };

        let name = format!("{}{}{} {}", prefix, expand_icon, icon, node.name);
        items.push((name, depth, node.selected));

        if node.expanded {
            for child in &node.children {
                self.flatten_tree_node(child, depth + 1, items);
            }
        }
    }

    /// Move tree selection up
    fn move_tree_selection_up(&mut self) {
        if self.tree_selection > 0 {
            self.tree_selection -= 1;
        }
    }

    /// Move tree selection down
    fn move_tree_selection_down(&mut self) {
        let tree_items = self.get_tree_items();
        if self.tree_selection < tree_items.len().saturating_sub(1) {
            self.tree_selection += 1;
        }
    }

    /// Select the current tree node
    fn select_current_node(&mut self) {
        let tree_items = self.get_tree_items();
        if self.tree_selection < tree_items.len() {
            if let Some(node_path) = self.get_node_path_at_index(self.tree_selection) {
                self.update_navigation_context(&node_path);
            }
        }
    }

    /// Expand the current node
    fn expand_current_node(&mut self) {
        if let Some(node_path) = self.get_node_path_at_index(self.tree_selection) {
            self.set_node_expanded(&node_path, true);
        }
    }

    /// Collapse the current node
    fn collapse_current_node(&mut self) {
        if let Some(node_path) = self.get_node_path_at_index(self.tree_selection) {
            self.set_node_expanded(&node_path, false);
        }
    }

    /// Get the path to a node at the given flattened index
    fn get_node_path_at_index(&self, index: usize) -> Option<Vec<usize>> {
        let mut current_index = 0;
        for (root_index, root_node) in self.tree_data.iter().enumerate() {
            if let Some(path) = self.find_node_path_recursive(root_node, index, &mut current_index, vec![root_index]) {
                return Some(path);
            }
        }
        None
    }

    /// Recursively find the path to a node at the given index
    fn find_node_path_recursive(&self, node: &TreeNode, target_index: usize, current_index: &mut usize, path: Vec<usize>) -> Option<Vec<usize>> {
        if *current_index == target_index {
            return Some(path);
        }
        *current_index += 1;

        if node.expanded {
            for (child_index, child) in node.children.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(child_index);
                if let Some(found_path) = self.find_node_path_recursive(child, target_index, current_index, child_path) {
                    return Some(found_path);
                }
            }
        }
        None
    }

    /// Set expansion state of a node at the given path
    fn set_node_expanded(&mut self, path: &[usize], expanded: bool) {
        if path.is_empty() {
            return;
        }

        let mut current_node = &mut self.tree_data[path[0]];
        for &index in &path[1..] {
            if index < current_node.children.len() {
                current_node = &mut current_node.children[index];
            } else {
                return;
            }
        }
        current_node.expanded = expanded;
    }

    /// Update navigation context based on the selected node path
    fn update_navigation_context(&mut self, path: &[usize]) {
        if path.is_empty() {
            return;
        }

        // Set domain from stored domain (since products are now root items)
        if let Some(domain) = &self.domain {
            self.navigation_context.domain = Some(domain.clone());
        }

        let mut current_node = &self.tree_data[path[0]];

        // Handle root node (which is now a product)
        if let crate::models::TreeNodeType::Product(product) = &current_node.node_type {
            self.navigation_context.product = Some(product.clone());
            self.navigation_context.project = None; // Reset project when selecting product
        }

        // Navigate to child nodes if any
        for &index in &path[1..] {
            if index < current_node.children.len() {
                current_node = &current_node.children[index];

                if let crate::models::TreeNodeType::Project(project) = &current_node.node_type {
                    self.navigation_context.project = Some(project.clone());
                }
            }
        }

        // Update selected state in tree
        self.clear_all_selections();
        self.set_node_selected(path, true);

        // Update command executor context
        self.command_executor.update_context(self.navigation_context.clone());
    }

    /// Clear all selections in the tree
    fn clear_all_selections(&mut self) {
        for root_node in &mut self.tree_data {
            Self::clear_selections_recursive(root_node);
        }
    }

    /// Recursively clear selections
    fn clear_selections_recursive(node: &mut TreeNode) {
        node.selected = false;
        for child in &mut node.children {
            Self::clear_selections_recursive(child);
        }
    }

    /// Set selection state of a node at the given path
    fn set_node_selected(&mut self, path: &[usize], selected: bool) {
        if path.is_empty() {
            return;
        }

        let mut current_node = &mut self.tree_data[path[0]];
        for &index in &path[1..] {
            if index < current_node.children.len() {
                current_node = &mut current_node.children[index];
            } else {
                return;
            }
        }
        current_node.selected = selected;
    }

    /// Handle input for command execution screen
    fn handle_command_execution_input(&mut self, code: KeyCode) -> Result<(), Box<dyn Error>> {
        use crate::command::CommandInputMode;

        match code {
            KeyCode::Backspace | KeyCode::Esc => {
                self.switch_screen(Screen::TreeNavigation);
            }
            KeyCode::Enter => {
                match self.command_input.mode {
                    CommandInputMode::SelectingCommand => {
                        // Select the current command
                        let available_commands = self.command_executor.get_available_commands();
                        if let Some(command) = available_commands.get(self.command_selection) {
                            self.command_input.set_command(command.clone());
                        }
                    }
                    CommandInputMode::TypingArgs => {
                        // Execute the command with current args
                        self.execute_selected_command()?;
                    }
                    CommandInputMode::Ready => {
                        // Execute the command
                        self.execute_selected_command()?;
                    }
                }
            }
            KeyCode::Up => {
                if self.command_input.mode == CommandInputMode::SelectingCommand
                    && self.command_selection > 0 {
                        self.command_selection -= 1;
                    }
            }
            KeyCode::Down => {
                if self.command_input.mode == CommandInputMode::SelectingCommand {
                    let available_commands = self.command_executor.get_available_commands();
                    if self.command_selection < available_commands.len().saturating_sub(1) {
                        self.command_selection += 1;
                    }
                }
            }
            KeyCode::Left => {
                if matches!(self.command_input.mode, CommandInputMode::TypingArgs) {
                    self.command_input.move_cursor_left();
                }
            }
            KeyCode::Right => {
                if matches!(self.command_input.mode, CommandInputMode::TypingArgs) {
                    self.command_input.move_cursor_right();
                }
            }
            KeyCode::Delete => {
                if matches!(self.command_input.mode, CommandInputMode::TypingArgs) {
                    self.command_input.delete_char();
                }
            }
            KeyCode::Char(c) => {
                match self.command_input.mode {
                    CommandInputMode::TypingArgs => {
                        self.command_input.insert_char(c);
                    }
                    CommandInputMode::SelectingCommand => {
                        // Quick selection by first letter
                        let available_commands = self.command_executor.get_available_commands();
                        for (i, command) in available_commands.iter().enumerate() {
                            if let AvailableCommand::Ctag { operation, .. } = command {
                                let first_char = operation.as_str().chars().next().unwrap_or(' ').to_ascii_lowercase();
                                if c.to_ascii_lowercase() == first_char {
                                    self.command_selection = i;
                                    break;
                                }
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
    fn execute_selected_command(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(selected_command) = &self.command_input.selected_command {
            if let AvailableCommand::Ctag { operation, .. } = selected_command {
                // Parse additional arguments from command input
                let args: Vec<String> = if self.command_input.text.trim().is_empty() {
                    Vec::new()
                } else {
                    self.command_input.text
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
                match self.command_executor.execute_command(command) {
                    Ok(result) => {
                        let status_msg = if result.success {
                            format!("Command executed successfully: {}", result.command)
                        } else {
                            format!("Command failed: {}", result.stderr)
                        };
                        self.ui.set_status(status_msg);
                    }
                    Err(e) => {
                        self.ui.set_status(format!("Error executing command: {e}"));
                    }
                }
            }
        }

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

    /// Enter search mode
    fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.filtered_tree_items = None;
        self.ui.set_status("Search mode: type to filter, Esc to exit".to_string());
    }

    /// Exit search mode
    fn exit_search_mode(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.filtered_tree_items = None;
        self.tree_selection = 0;
        self.ui.set_status("Ready".to_string());
    }

    /// Handle search input
    fn handle_search_input(&mut self, code: KeyCode) -> Result<(), Box<dyn Error>> {
        match code {
            KeyCode::Esc => {
                self.exit_search_mode();
            }
            KeyCode::Enter => {
                // Exit search mode but keep filter
                if !self.search_query.is_empty() {
                    self.search_mode = false;
                    self.ui.set_status(format!("Filtered by: '{}'", self.search_query));
                } else {
                    self.exit_search_mode();
                }
            }
            KeyCode::Backspace => {
                if !self.search_query.is_empty() {
                    self.search_query.pop();
                    self.update_search_filter();
                }
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.update_search_filter();
            }
            KeyCode::Up => {
                if let Some(ref filtered_items) = self.filtered_tree_items {
                    if self.tree_selection > 0 {
                        self.tree_selection -= 1;
                    }
                } else {
                    self.move_tree_selection_up();
                }
            }
            KeyCode::Down => {
                if let Some(ref filtered_items) = self.filtered_tree_items {
                    if self.tree_selection < filtered_items.len().saturating_sub(1) {
                        self.tree_selection += 1;
                    }
                } else {
                    self.move_tree_selection_down();
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Update search filter
    fn update_search_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_tree_items = None;
            self.tree_selection = 0;
            return;
        }

        let all_items = self.get_tree_items();
        let query_lower = self.search_query.to_lowercase();

        let filtered: Vec<(String, usize, bool)> = all_items
            .into_iter()
            .filter(|(name, _depth, _selected)| {
                name.to_lowercase().contains(&query_lower)
            })
            .collect();

        self.filtered_tree_items = Some(filtered);
        self.tree_selection = 0;
    }

    /// Get the items to display (either filtered or full tree)
    pub fn get_display_items(&self) -> Vec<(String, usize, bool)> {
        if let Some(ref filtered) = self.filtered_tree_items {
            filtered.clone()
        } else {
            self.get_tree_items()
        }
    }


}

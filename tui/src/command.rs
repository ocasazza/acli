//! Command execution system for the TUI

use crate::models::{NavigationContext, ProductType, TreeNode, TreeNodeType};
use crate::tree_navigation::TreeNavigationManager;
use nix_rust_template::{execute_ctag_operation, CtagOperation, CtagResult, CtagPageResult};
use std::error::Error;

/// Represents a command that can be executed in the TUI
#[derive(Debug, Clone)]
pub struct TuiCommand {
    /// Command name (e.g., "ctag")
    pub name: String,
    /// Command operation (e.g., "list", "add", "remove")
    pub operation: String,
    /// Additional arguments
    pub args: Vec<String>,
    /// Whether this is a dry run
    pub dry_run: bool,
}

/// Result of executing a command
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Exit code of the command
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Command that was executed
    pub command: String,
    /// Whether the command was successful
    pub success: bool,
}

/// Available commands for different products
#[derive(Debug, Clone)]
pub enum AvailableCommand {
    /// ctag command for Confluence
    Ctag {
        operation: TuiCtagOperation,
        description: String,
    },
}

/// ctag operations for TUI
#[derive(Debug, Clone)]
pub enum TuiCtagOperation {
    List,
    Add,
    Update,
    Remove,
}

impl TuiCtagOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            TuiCtagOperation::List => "list",
            TuiCtagOperation::Add => "add",
            TuiCtagOperation::Update => "update",
            TuiCtagOperation::Remove => "remove",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TuiCtagOperation::List => "List labels for pages matching CQL",
            TuiCtagOperation::Add => "Add labels to pages matching CQL",
            TuiCtagOperation::Update => "Update labels on pages matching CQL",
            TuiCtagOperation::Remove => "Remove labels from pages matching CQL",
        }
    }
}

/// Command execution engine
pub struct CommandExecutor {
    /// Current navigation context
    context: NavigationContext,
    /// Command history
    pub history: Vec<CommandResult>,
}

impl CommandExecutor {
    pub fn new(context: NavigationContext) -> Self {
        Self {
            context,
            history: Vec::new(),
        }
    }

    /// Update the navigation context
    pub fn update_context(&mut self, context: NavigationContext) {
        self.context = context;
    }

    /// Get available commands for the current context
    pub fn get_available_commands(&self) -> Vec<AvailableCommand> {
        let mut commands = Vec::new();

        if let (Some(_), Some(product), Some(_)) = (
            &self.context.domain,
            &self.context.product,
            &self.context.project,
        ) {
            match product.product_type {
                ProductType::Confluence => {
                    commands.push(AvailableCommand::Ctag {
                        operation: TuiCtagOperation::List,
                        description: "List labels for pages in this space".to_string(),
                    });
                    commands.push(AvailableCommand::Ctag {
                        operation: TuiCtagOperation::Add,
                        description: "Add labels to pages in this space".to_string(),
                    });
                    commands.push(AvailableCommand::Ctag {
                        operation: TuiCtagOperation::Update,
                        description: "Update labels on pages in this space".to_string(),
                    });
                    commands.push(AvailableCommand::Ctag {
                        operation: TuiCtagOperation::Remove,
                        description: "Remove labels from pages in this space".to_string(),
                    });
                }
                ProductType::Jira | ProductType::Jsm => {
                    // Future: Add Jira/JSM commands here
                }
            }
        }

        commands
    }

    /// Execute a command using the shared library directly
    pub fn execute_command(
        &mut self,
        command: TuiCommand,
        label_buffer: Option<&[String]>,
    ) -> Result<CommandResult, Box<dyn Error>> {
        if command.name != "ctag" {
            return Err(format!("Unknown command: {}", command.name).into());
        }

        let cql_context = self
            .context
            .cql_context()
            .ok_or("No valid context for command execution")?;

        // Use label buffer if available and no explicit args provided
        let tags_arg = if command.args.is_empty() {
            if let Some(labels) = label_buffer {
                if !labels.is_empty() {
                    Some(labels.join(","))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            command.args.get(0).cloned()
        };

        // Convert TUI command to shared library operation
        let operation = match command.operation.as_str() {
            "list" => CtagOperation::List {
                cql: cql_context.clone(),
                tags: tags_arg.clone(),
                tree: command.args.contains(&"--tree".to_string()),
            },
            "add" => CtagOperation::Add {
                cql: cql_context.clone(),
                tags: tags_arg.unwrap_or_default(),
            },
            "update" => CtagOperation::Update {
                cql: cql_context.clone(),
                tags: tags_arg.unwrap_or_default(),
            },
            "remove" => CtagOperation::Remove {
                cql: cql_context.clone(),
                tags: tags_arg.unwrap_or_default(),
            },
            _ => return Err(format!("Unknown operation: {}", command.operation).into()),
        };

        // Execute using shared library
        let ctag_result = execute_ctag_operation(operation, command.dry_run, false)?;

        // Convert CtagResult to CommandResult
        let result = CommandResult {
            exit_code: if ctag_result.success { 0 } else { 1 },
            stdout: format_ctag_result_for_display(&ctag_result),
            stderr: ctag_result.error.unwrap_or_default(),
            command: format!("ctag {} \"{}\" {}", command.operation, cql_context, command.args.join(" ")),
            success: ctag_result.success,
        };

        // Add to history
        self.history.push(result.clone());

        Ok(result)
    }


    /// Get the most recent command result
    pub fn get_last_result(&self) -> Option<&CommandResult> {
        self.history.last()
    }

    /// Clear command history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Execute command and return both result and tree structure
    pub fn execute_command_with_tree(
        &mut self,
        command: TuiCommand,
    ) -> Result<(CommandResult, Option<TreeNavigationManager>), Box<dyn Error>> {
        if command.name != "ctag" {
            return Err(format!("Unknown command: {}", command.name).into());
        }

        let cql_context = self
            .context
            .cql_context()
            .ok_or("No valid context for command execution")?;

        // Convert TUI command to shared library operation
        let operation = match command.operation.as_str() {
            "list" => CtagOperation::List {
                cql: cql_context.clone(),
                tags: command.args.get(0).cloned(),
                tree: command.args.contains(&"--tree".to_string()),
            },
            "add" => CtagOperation::Add {
                cql: cql_context.clone(),
                tags: command.args.get(0).cloned().unwrap_or_default(),
            },
            "update" => CtagOperation::Update {
                cql: cql_context.clone(),
                tags: command.args.get(0).cloned().unwrap_or_default(),
            },
            "remove" => CtagOperation::Remove {
                cql: cql_context.clone(),
                tags: command.args.get(0).cloned().unwrap_or_default(),
            },
            _ => return Err(format!("Unknown operation: {}", command.operation).into()),
        };

        // Execute using shared library
        let ctag_result = execute_ctag_operation(operation, command.dry_run, false)?;

        // Create tree structure for results if successful
        let tree_manager = if ctag_result.success && !ctag_result.pages.is_empty() {
            Some(create_tree_from_ctag_results(&ctag_result))
        } else {
            None
        };

        // Convert CtagResult to CommandResult
        let result = CommandResult {
            exit_code: if ctag_result.success { 0 } else { 1 },
            stdout: format_ctag_result_for_display(&ctag_result),
            stderr: ctag_result.error.unwrap_or_default(),
            command: format!("ctag {} \"{}\" {}", command.operation, cql_context, command.args.join(" ")),
            success: ctag_result.success,
        };

        // Add to history
        self.history.push(result.clone());

        Ok((result, tree_manager))
    }
}

/// Convert CtagResult to TreeNavigationManager for display
pub fn create_tree_from_ctag_results(result: &CtagResult) -> TreeNavigationManager {
    let mut tree_manager = TreeNavigationManager::new();
    let mut tree_nodes = Vec::new();

    for page in &result.pages {
        let tree_node = convert_page_to_tree_node(page);
        tree_nodes.push(tree_node);
    }

    tree_manager.tree_data = tree_nodes;

    // Expand all nodes by default for better visibility
    expand_all_nodes(&mut tree_manager.tree_data);

    tree_manager
}

/// Recursively expand all nodes in the tree
fn expand_all_nodes(nodes: &mut [TreeNode]) {
    for node in nodes {
        node.expanded = true;
        expand_all_nodes(&mut node.children);
    }
}

/// Convert a CtagPageResult to a TreeNode
fn convert_page_to_tree_node(page: &CtagPageResult) -> TreeNode {
    // Create a dummy project to satisfy TreeNodeType::Project requirements
    let _project = nix_rust_template::Project {
        id: page.id.clone(),
        name: page.title.clone(),
        key: page.id.clone(), // Use ID as key since we don't have a real key
        description: Some(format!("Labels: {}", page.labels.join(", "))),
        project_type: "page".to_string(),
        homepage_id: None, // Pages don't have homepage IDs
    };

    let mut tree_node = TreeNode {
        name: page.title.clone(),
        node_type: TreeNodeType::Page {
            id: page.id.clone(),
            labels: page.labels.clone(),
        },
        expanded: false,
        selected: false,
        children: Vec::new(),
        children_loaded: page.children.is_empty(), // If no children, they're "loaded"
        has_children: !page.children.is_empty(),
        loading_children: false,
    };

    // Convert children recursively
    for child in &page.children {
        tree_node.children.push(convert_page_to_tree_node(child));
    }

    tree_node
}

/// Format CtagResult for display in TUI
fn format_ctag_result_for_display(result: &CtagResult) -> String {
    let mut output = Vec::new();

    // Add summary
    output.push(result.summary.clone());

    if !result.pages.is_empty() {
        output.push("".to_string()); // Empty line

        // Format pages based on whether they have children (tree format)
        let has_tree = result.pages.iter().any(|p| !p.children.is_empty());

        if has_tree {
            output.push("Pages matching CQL query:".to_string());
            for (i, page) in result.pages.iter().enumerate() {
                let is_last = i == result.pages.len() - 1;
                format_page_tree_display(page, "", is_last, &mut output);
            }
        } else {
            // Flat list format
            for page in &result.pages {
                let display_name = if page.highlighted {
                    format!("* {}", page.title) // Simple highlight with asterisk
                } else {
                    page.title.clone()
                };

                if !page.labels.is_empty() {
                    output.push(format!("{} [{}]", display_name, page.labels.join(", ")));
                } else {
                    output.push(display_name);
                }
            }
        }
    }

    output.join("\n")
}

/// Format a page and its children for tree display
fn format_page_tree_display(
    page: &CtagPageResult,
    prefix: &str,
    is_last: bool,
    output: &mut Vec<String>,
) {
    let tree_symbol = if is_last { "└── " } else { "├── " };

    let display_name = if page.highlighted {
        format!("* {}", page.title) // Simple highlight with asterisk
    } else {
        page.title.clone()
    };

    if !page.labels.is_empty() {
        output.push(format!(
            "{}{}{} [{}]",
            prefix,
            tree_symbol,
            display_name,
            page.labels.join(", ")
        ));
    } else {
        output.push(format!("{}{}{}", prefix, tree_symbol, display_name));
    }

    // Display children
    if !page.children.is_empty() {
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        for (i, child) in page.children.iter().enumerate() {
            let is_last_child = i == page.children.len() - 1;
            format_page_tree_display(child, &new_prefix, is_last_child, output);
        }
    }
}

/// Command input state for the TUI
#[derive(Debug, Clone)]
pub struct CommandInput {
    /// Current input text
    pub text: String,
    /// Cursor position
    pub cursor: usize,
    /// Selected command type
    pub selected_command: Option<AvailableCommand>,
    /// Input mode (typing args, selecting command, etc.)
    pub mode: CommandInputMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandInputMode {
    SelectingCommand,
    TypingArgs,
    Ready,
}

impl Default for CommandInput {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandInput {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            selected_command: None,
            mode: CommandInputMode::SelectingCommand,
        }
    }

    /// Insert character at cursor position
    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.text.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
        }
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    /// Set selected command
    pub fn set_command(&mut self, command: AvailableCommand) {
        self.selected_command = Some(command);
        self.mode = CommandInputMode::TypingArgs;
        self.clear();
    }

    /// Reset to command selection mode
    pub fn reset_to_selection(&mut self) {
        self.selected_command = None;
        self.mode = CommandInputMode::SelectingCommand;
        self.clear();
    }
}

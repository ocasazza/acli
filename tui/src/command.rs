//! Command execution system for the TUI

use crate::models::{NavigationContext, ProductType};
use std::error::Error;
use std::process::Command;

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
        operation: CtagOperation,
        description: String,
    },
}

/// ctag operations
#[derive(Debug, Clone)]
pub enum CtagOperation {
    List,
    Add,
    Update,
    Remove,
}

impl CtagOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            CtagOperation::List => "list",
            CtagOperation::Add => "add",
            CtagOperation::Update => "update",
            CtagOperation::Remove => "remove",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CtagOperation::List => "List labels for pages matching CQL",
            CtagOperation::Add => "Add labels to pages matching CQL",
            CtagOperation::Update => "Update labels on pages matching CQL",
            CtagOperation::Remove => "Remove labels from pages matching CQL",
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

        if let (Some(_), Some(product), Some(_)) = (&self.context.domain, &self.context.product, &self.context.project) {
            match product.product_type {
                ProductType::Confluence => {
                    commands.push(AvailableCommand::Ctag {
                        operation: CtagOperation::List,
                        description: "List labels for pages in this space".to_string(),
                    });
                    commands.push(AvailableCommand::Ctag {
                        operation: CtagOperation::Add,
                        description: "Add labels to pages in this space".to_string(),
                    });
                    commands.push(AvailableCommand::Ctag {
                        operation: CtagOperation::Update,
                        description: "Update labels on pages in this space".to_string(),
                    });
                    commands.push(AvailableCommand::Ctag {
                        operation: CtagOperation::Remove,
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

    /// Execute a command
    pub fn execute_command(&mut self, command: TuiCommand) -> Result<CommandResult, Box<dyn Error>> {
        let cmd_string = self.build_command_string(&command)?;

        // Execute the command using the acli binary
        let output = Command::new("cargo")
            .args(["run", "--bin", "acli", "--"])
            .args(self.parse_command_args(&cmd_string))
            .output()?;

        let result = CommandResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            command: cmd_string,
            success: output.status.success(),
        };

        // Add to history
        self.history.push(result.clone());

        Ok(result)
    }

    /// Build the command string with context
    fn build_command_string(&self, command: &TuiCommand) -> Result<String, Box<dyn Error>> {
        match command.name.as_str() {
            "ctag" => {
                let cql_context = self.context.cql_context()
                    .ok_or("No valid context for command execution")?;

                let mut cmd_parts = vec!["ctag".to_string(), command.operation.clone()];

                // Add CQL context
                cmd_parts.push(format!("\"{cql_context}\""));

                // Add additional arguments
                cmd_parts.extend(command.args.clone());

                // Add flags
                if command.dry_run {
                    cmd_parts.push("--dry-run".to_string());
                }

                Ok(cmd_parts.join(" "))
            }
            _ => Err(format!("Unknown command: {}", command.name).into()),
        }
    }

    /// Parse command string into arguments
    fn parse_command_args(&self, cmd_string: &str) -> Vec<String> {
        // Simple argument parsing - split by spaces but respect quotes
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut in_quotes = false;
        let chars = cmd_string.chars();

        for c in chars {
            match c {
                '"' => in_quotes = !in_quotes,
                ' ' if !in_quotes => {
                    if !current_arg.is_empty() {
                        args.push(current_arg.clone());
                        current_arg.clear();
                    }
                }
                _ => current_arg.push(c),
            }
        }

        if !current_arg.is_empty() {
            args.push(current_arg);
        }

        args
    }

    /// Get the most recent command result
    pub fn get_last_result(&self) -> Option<&CommandResult> {
        self.history.last()
    }

    /// Clear command history
    pub fn clear_history(&mut self) {
        self.history.clear();
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

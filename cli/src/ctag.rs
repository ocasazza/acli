use clap::{Args, Subcommand};
use nix_rust_template::{execute_ctag_operation, CtagOperation, CtagPageResult};
use std::error::Error;

/// ctag command: operate on Confluence page labels matched by a CQL expression.
///
/// Intended usage examples:
///   acli ctag list "parent = 1234" --tree
///   acli ctag add "parent = 1234" "foo,bar,baz"
///   acli ctag update "parent = 1234" "foo:bar,baz:foo"
///   acli ctag remove "parent = 1234" "foo,bar,baz"
///
/// This module provides a CLI-friendly struct and a `run` function that applies
/// the requested operation against one or more provided ConfluencePageTree
/// instances. The functions are intentionally stubbed — they operate on the
/// in-memory trees and record actions via the shared library API surface.
#[derive(Args, Debug)]
pub struct CtagCmd {
    #[command(subcommand)]
    pub operation: CtagOp,
}

#[derive(Subcommand, Debug)]
pub enum CtagOp {
    /// List labels for pages matching the CQL expression
    List {
        /// CQL expression selecting pages to operate on
        cql: String,
        /// Optional comma-separated list of tags to highlight in the output
        tags: Option<String>,
        /// Display results in tree format
        #[arg(long)]
        tree: bool,
    },
    /// Add labels to pages matching the CQL expression
    Add {
        /// CQL expression selecting pages to operate on
        cql: String,
        /// Comma-separated list of labels to add (e.g., "foo,bar,baz")
        tags: String,
    },
    /// Update labels on pages matching the CQL expression
    Update {
        /// CQL expression selecting pages to operate on
        cql: String,
        /// Comma-separated list of label updates in format "old:new,old2:new2" (e.g., "foo:bar,baz:foo")
        tags: String,
    },
    /// Remove labels from pages matching the CQL expression
    Remove {
        /// CQL expression selecting pages to operate on
        cql: String,
        /// Comma-separated list of labels to remove (e.g., "foo,bar,baz")
        tags: String,
    },
}

/// Execute the ctag command using the shared library interface.
pub fn run(
    cmd: &CtagCmd,
    dry_run: bool,
    _pretty: bool,
    verbose: bool,
) -> Result<(), Box<dyn Error>> {
    // Convert CLI args to shared library operation
    let operation = match &cmd.operation {
        CtagOp::List { cql, tags, tree } => CtagOperation::List {
            cql: cql.clone(),
            tags: tags.clone(),
            tree: *tree,
        },
        CtagOp::Add { cql, tags } => CtagOperation::Add {
            cql: cql.clone(),
            tags: tags.clone(),
        },
        CtagOp::Update { cql, tags } => CtagOperation::Update {
            cql: cql.clone(),
            tags: tags.clone(),
        },
        CtagOp::Remove { cql, tags } => CtagOperation::Remove {
            cql: cql.clone(),
            tags: tags.clone(),
        },
    };
    // Execute operation using shared library
    let result = execute_ctag_operation(operation, dry_run, verbose)?;
    // Format and display results for CLI
    if !result.success {
        if let Some(error) = &result.error {
            eprintln!("Error: {}", error);
        }
        return Err("Operation failed".into());
    }
    // Print summary
    println!("{}", result.summary);
    // Display pages based on operation type
    match &cmd.operation {
        CtagOp::List { tree, .. } => {
            if *tree {
                display_pages_tree_from_result(&result.pages)?;
            } else {
                display_pages_flat_from_result(&result.pages)?;
            }
        }
        _ => {
            // For non-list operations, show affected pages
            if !result.pages.is_empty() {
                println!("\nAffected pages:");
                for page in &result.pages {
                    println!("  - {}", page.title);
                }
            }
        }
    }
    Ok(())
}


/// Display pages from CtagResult in tree format
fn display_pages_tree_from_result(pages: &[CtagPageResult]) -> Result<(), Box<dyn Error>> {
    if pages.is_empty() {
        println!("No pages found.");
        return Ok(());
    }
    println!("Pages matching CQL query:");
    for (i, page) in pages.iter().enumerate() {
        let is_last = i == pages.len() - 1;
        display_page_result_with_children(page, "", is_last)?;
    }
    Ok(())
}

/// Display a single page result and its children recursively
fn display_page_result_with_children(
    page: &CtagPageResult,
    prefix: &str,
    is_last: bool,
) -> Result<(), Box<dyn Error>> {
    let tree_symbol = if is_last { "└── " } else { "├── " };
    let display_name = if page.highlighted {
        format!("\x1b[1;33m{}\x1b[0m", page.title) // Yellow highlight
    } else {
        page.title.clone()
    };
    if !page.labels.is_empty() {
        println!(
            "{}{}{} [{}]",
            prefix,
            tree_symbol,
            display_name,
            page.labels.join(", ")
        );
    } else {
        println!("{prefix}{tree_symbol}{display_name}");
    }
    // Display children
    if !page.children.is_empty() {
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        for (i, child) in page.children.iter().enumerate() {
            let is_last_child = i == page.children.len() - 1;
            display_page_result_with_children(child, &new_prefix, is_last_child)?;
        }
    }
    Ok(())
}

/// Display pages from CtagResult in flat format
fn display_pages_flat_from_result(pages: &[CtagPageResult]) -> Result<(), Box<dyn Error>> {
    if pages.is_empty() {
        println!("No pages found.");
        return Ok(());
    }
    for page in pages {
        let display_name = if page.highlighted {
            format!("\x1b[1;33m{}\x1b[0m", page.title) // Yellow highlight
        } else {
            page.title.clone()
        };

        if !page.labels.is_empty() {
            println!("{} [{}]", display_name, page.labels.join(", "));
        } else {
            println!("{display_name}");
        }
    }
    Ok(())
}

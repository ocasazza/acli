use clap::{Args, Subcommand};
use nix_rust_template::{ConfluenceClient, ConfluenceConfig, ConfluencePage};
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

/// Execute the ctag command against the provided Confluence data.
///
/// The implementation is intentionally minimal and uses the stubbed shared
/// library APIs (in-memory) to demonstrate the integration surface. The CQL
/// expression may match multiple page trees; this function will execute the
/// requested operation on every tree in the provided slice.
///
/// Page list is a bit different than the other operations. For all pages matched by CQL expression, recurse the page and highlight the output of any pages that match the supplied tags. We show **all** pages that match the CQL and highlight anything within the CQL outputted that contains a tag. When --tree is supplied, the output should be formatted as a tree, similar to the tree unix command, also with pages with matching labels highlighted
///
/// Other CtagOps simply fetch the CQL results and apply the operations supplied to each result.
pub fn run(
    cmd: &CtagCmd,
    dry_run: bool,
    _pretty: bool,
    verbose: bool,
) -> Result<(), Box<dyn Error>> {
    match &cmd.operation {
        CtagOp::List { cql, tags, tree } => {
            if verbose {
                eprintln!("Listing pages matching: {cql}");
                if let Some(highlight_tags) = tags {
                    eprintln!("Highlighting pages with tags: {highlight_tags}");
                }
                if *tree {
                    eprintln!("Using tree format");
                }
            }
            // Parse highlight tags if provided
            let highlight_list: Option<Vec<&str>> = tags
                .as_ref()
                .map(|t: &String| t.split(',').map(|s: &str| s.trim()).collect());
            if dry_run {
                println!("DRY RUN: Would list pages for CQL: {cql}");
                if let Some(tags) = &highlight_list {
                    println!("DRY RUN: Would highlight pages with tags: {tags:?}");
                }
                if *tree {
                    println!("DRY RUN: Would use tree format");
                }
            } else {
                // Create Confluence client and execute query
                let client = create_confluence_client()?;
                let pages = client.query_pages_by_cql(cql)?;

                if *tree {
                    display_pages_tree(&pages, highlight_list.as_deref())?;
                } else {
                    display_pages_flat(&pages, highlight_list.as_deref())?;
                }
            }
        }
        CtagOp::Add { cql, tags } => {
            if verbose {
                eprintln!("Adding labels '{tags}' to pages matching: {cql}");
            }
            // Parse comma-separated tags
            let tag_list: Vec<&str> = tags.split(',').map(|s: &str| s.trim()).collect();

            if dry_run {
                println!("DRY RUN: Would add labels {tag_list:?} to pages matching CQL: {cql}");
            } else {
                // Create Confluence client and execute query
                let client = create_confluence_client()?;
                let pages = client.query_pages_by_cql(cql)?;

                if pages.is_empty() {
                    println!("No pages found matching CQL: {cql}");
                    return Ok(());
                }

                println!("Adding labels {:?} to {} pages...", tag_list, pages.len());

                // Extract page IDs for bulk operation
                let page_ids: Vec<&str> = pages.iter().map(|p| p.id.as_str()).collect();

                // Use bulk operation for efficiency
                client.bulk_add_labels(&page_ids, &tag_list)?;

                println!("Successfully added labels to {} pages:", pages.len());
                for page in &pages {
                    println!("  - {}", page.title);
                }
            }
        }
        CtagOp::Update { cql, tags } => {
            if verbose {
                eprintln!("Updating labels '{tags}' on pages matching: {cql}");
            }
            // Parse colon-separated tag updates
            let updates: Result<Vec<(String, String)>, Box<dyn Error>> = tags
                .split(',')
                .map(|s: &str| s.trim())
                .map(|update: &str| -> Result<(String, String), Box<dyn Error>> {
                    let parts: Vec<&str> = update.split(':').collect();
                    if parts.len() != 2 {
                        return Err(format!(
                            "Invalid update format '{update}'. Expected 'old:new'"
                        )
                        .into());
                    }
                    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
                })
                .collect();

            let updates = updates?;

            if dry_run {
                println!("DRY RUN: Would update labels {updates:?} on pages matching CQL: {cql}");
            } else {
                // Create Confluence client and execute query
                let client = create_confluence_client()?;
                let pages = client.query_pages_by_cql(cql)?;

                if pages.is_empty() {
                    println!("No pages found matching CQL: {cql}");
                    return Ok(());
                }

                println!("Updating labels {:?} on {} pages...", updates, pages.len());

                // Extract page IDs for bulk operation
                let page_ids: Vec<&str> = pages.iter().map(|p| p.id.as_str()).collect();

                // Use bulk operation for efficiency
                client.bulk_update_labels(&page_ids, &updates)?;

                println!("Successfully updated labels on {} pages:", pages.len());
                for page in &pages {
                    println!("  - {}", page.title);
                }
            }
        }
        CtagOp::Remove { cql, tags } => {
            if verbose {
                eprintln!("Removing labels '{tags}' from pages matching: {cql}");
            }
            // Parse comma-separated tags
            let tag_list: Vec<&str> = tags.split(',').map(|s: &str| s.trim()).collect();

            if dry_run {
                println!(
                    "DRY RUN: Would remove labels {tag_list:?} from pages matching CQL: {cql}"
                );
            } else {
                // Create Confluence client and execute query
                let client = create_confluence_client()?;
                let pages = client.query_pages_by_cql(cql)?;

                if pages.is_empty() {
                    println!("No pages found matching CQL: {cql}");
                    return Ok(());
                }

                println!(
                    "Removing labels {:?} from {} pages...",
                    tag_list,
                    pages.len()
                );

                // Extract page IDs for bulk operation
                let page_ids: Vec<&str> = pages.iter().map(|p| p.id.as_str()).collect();

                // Use bulk operation for efficiency
                client.bulk_remove_labels(&page_ids, &tag_list)?;

                println!("Successfully removed labels from {} pages:", pages.len());
                for page in &pages {
                    println!("  - {}", page.title);
                }
            }
        }
    }
    Ok(())
}

/// Create a Confluence client using environment variables.
fn create_confluence_client() -> Result<ConfluenceClient, Box<dyn Error>> {
    dotenv::dotenv().ok(); // Load .env file, ignore if not found

    let base_url =
        std::env::var("ATLASSIAN_URL").map_err(|_| "ATLASSIAN_URL environment variable not set")?;
    let username = std::env::var("ATLASSIAN_USERNAME")
        .map_err(|_| "ATLASSIAN_USERNAME environment variable not set")?;
    let api_token = std::env::var("ATLASSIAN_TOKEN")
        .map_err(|_| "ATLASSIAN_TOKEN environment variable not set")?;

    let config = ConfluenceConfig {
        base_url,
        username,
        api_token,
    };

    ConfluenceClient::new(config).map_err(|e| e.into())
}

/// Display pages in a tree format similar to the unix tree command.
fn display_pages_tree(
    pages: &[ConfluencePage],
    highlight_tags: Option<&[&str]>,
) -> Result<(), Box<dyn Error>> {
    if pages.is_empty() {
        println!("No pages found.");
        return Ok(());
    }

    // Create a client to fetch child pages for each page
    let client = create_confluence_client()?;

    println!("Pages matching CQL query:");

    for (i, page) in pages.iter().enumerate() {
        let is_last = i == pages.len() - 1;
        display_page_with_children(&client, page, "", is_last, highlight_tags)?;
    }

    Ok(())
}

/// Display a page and recursively fetch and display its children
fn display_page_with_children(
    client: &ConfluenceClient,
    page: &ConfluencePage,
    prefix: &str,
    is_last: bool,
    highlight_tags: Option<&[&str]>,
) -> Result<(), Box<dyn Error>> {
    let tree_symbol = if is_last { "└── " } else { "├── " };
    let labels = get_page_labels(page);

    let display_name = if should_highlight_page(&labels, highlight_tags) {
        format!("\x1b[1;33m{}\x1b[0m", page.title) // Yellow highlight
    } else {
        page.title.clone()
    };

    if !labels.is_empty() {
        println!(
            "{}{}{} [{}]",
            prefix,
            tree_symbol,
            display_name,
            labels.join(", ")
        );
    } else {
        println!("{prefix}{tree_symbol}{display_name}");
    }

    // Fetch child pages for this page
    let child_cql = format!("parent = {}", page.id);
    if let Ok(children) = client.query_pages_by_cql(&child_cql) {
        if !children.is_empty() {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            for (i, child) in children.iter().enumerate() {
                let is_last_child = i == children.len() - 1;
                display_page_with_children(
                    client,
                    child,
                    &new_prefix,
                    is_last_child,
                    highlight_tags,
                )?;
            }
        }
    }

    Ok(())
}

/// Display pages in a flat list format.
fn display_pages_flat(
    pages: &[ConfluencePage],
    highlight_tags: Option<&[&str]>,
) -> Result<(), Box<dyn Error>> {
    if pages.is_empty() {
        println!("No pages found.");
        return Ok(());
    }

    for page in pages {
        let labels = get_page_labels(page);

        let display_name = if should_highlight_page(&labels, highlight_tags) {
            format!("\x1b[1;33m{}\x1b[0m", page.title) // Yellow highlight
        } else {
            page.title.clone()
        };

        if !labels.is_empty() {
            println!("{} [{}]", display_name, labels.join(", "));
        } else {
            println!("{display_name}");
        }
    }

    Ok(())
}

/// Extract labels from a page's metadata.
fn get_page_labels(page: &ConfluencePage) -> Vec<String> {
    page.metadata
        .as_ref()
        .and_then(|m| m.labels.as_ref())
        .map(|labels| labels.results.iter().map(|l| l.name.clone()).collect())
        .unwrap_or_default()
}

/// Check if a page should be highlighted based on its labels.
fn should_highlight_page(page_labels: &[String], highlight_tags: Option<&[&str]>) -> bool {
    if let Some(tags) = highlight_tags {
        page_labels
            .iter()
            .any(|label| tags.contains(&label.as_str()))
    } else {
        false
    }
}

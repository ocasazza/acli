//! Shared ctag functionality that returns structured data instead of printing to stdout

use crate::{ConfluenceClient, ConfluenceConfig, ConfluencePage};
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Structured result from ctag operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtagResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Operation that was performed
    pub operation: String,
    /// CQL query that was used
    pub cql: String,
    /// Pages that were affected/found
    pub pages: Vec<CtagPageResult>,
    /// Error message if operation failed
    pub error: Option<String>,
    /// Summary message
    pub summary: String,
}

/// Page result with label information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtagPageResult {
    /// Page ID
    pub id: String,
    /// Page title
    pub title: String,
    /// Current labels on the page
    pub labels: Vec<String>,
    /// Whether this page should be highlighted (for list operations)
    pub highlighted: bool,
    /// Child pages (for tree display)
    pub children: Vec<CtagPageResult>,
}

/// ctag operation types
#[derive(Debug, Clone)]
pub enum CtagOperation {
    /// List labels for pages matching the CQL expression
    List {
        /// CQL expression selecting pages to operate on
        cql: String,
        /// Optional comma-separated list of tags to highlight in the output
        tags: Option<String>,
        /// Display results in tree format
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

/// Execute ctag operation and return structured data
pub fn execute_ctag_operation(
    operation: CtagOperation,
    dry_run: bool,
    verbose: bool,
) -> Result<CtagResult, Box<dyn Error>> {
    match operation {
        CtagOperation::List { cql, tags, tree } => {
            execute_list_operation(cql, tags, tree, verbose)
        }
        CtagOperation::Add { cql, tags } => {
            execute_add_operation(cql, tags, dry_run, verbose)
        }
        CtagOperation::Update { cql, tags } => {
            execute_update_operation(cql, tags, dry_run, verbose)
        }
        CtagOperation::Remove { cql, tags } => {
            execute_remove_operation(cql, tags, dry_run, verbose)
        }
    }
}

/// Execute list operation
fn execute_list_operation(
    cql: String,
    tags: Option<String>,
    tree: bool,
    _verbose: bool,
) -> Result<CtagResult, Box<dyn Error>> {
    let client = create_confluence_client()?;
    let pages = client.query_pages_by_cql(&cql)?;

    // Parse highlight tags if provided
    let highlight_list: Option<Vec<&str>> = tags
        .as_ref()
        .map(|t| t.split(',').map(|s| s.trim()).collect());

    let mut page_results = Vec::new();

    for page in &pages {
        let labels = get_page_labels(page);
        let highlighted = should_highlight_page(&labels, highlight_list.as_deref());

        let page_result = CtagPageResult {
            id: page.id.clone(),
            title: page.title.clone(),
            labels,
            highlighted,
            children: Vec::new(),
        };

        // Don't fetch children here - we'll build the tree from all pages at once

        page_results.push(page_result);
    }

    // If tree format requested, organize pages into tree structure
    let final_pages = if tree {
        build_tree_from_pages(page_results, &pages)
    } else {
        page_results
    };

    Ok(CtagResult {
        success: true,
        operation: "list".to_string(),
        cql: cql.clone(),
        pages: final_pages,
        error: None,
        summary: format!("Found {} pages matching CQL: {}", pages.len(), cql),
    })
}

/// Execute add operation
fn execute_add_operation(
    cql: String,
    tags: String,
    dry_run: bool,
    _verbose: bool,
) -> Result<CtagResult, Box<dyn Error>> {
    let client = create_confluence_client()?;
    let pages = client.query_pages_by_cql(&cql)?;

    let tag_list: Vec<&str> = tags.split(',').map(|s| s.trim()).collect();

    if pages.is_empty() {
        return Ok(CtagResult {
            success: true,
            operation: "add".to_string(),
            cql: cql.clone(),
            pages: Vec::new(),
            error: None,
            summary: format!("No pages found matching CQL: {}", cql),
        });
    }

    let page_results: Vec<CtagPageResult> = pages.iter().map(|page| {
        let labels = get_page_labels(page);
        CtagPageResult {
            id: page.id.clone(),
            title: page.title.clone(),
            labels,
            highlighted: false,
            children: Vec::new(),
        }
    }).collect();

    if !dry_run {
        let page_ids: Vec<&str> = pages.iter().map(|p| p.id.as_str()).collect();
        client.bulk_add_labels(&page_ids, &tag_list)?;
    }

    let summary = if dry_run {
        format!("DRY RUN: Would add labels {:?} to {} pages", tag_list, pages.len())
    } else {
        format!("Successfully added labels {:?} to {} pages", tag_list, pages.len())
    };

    Ok(CtagResult {
        success: true,
        operation: "add".to_string(),
        cql: cql.clone(),
        pages: page_results,
        error: None,
        summary,
    })
}

/// Execute update operation
fn execute_update_operation(
    cql: String,
    tags: String,
    dry_run: bool,
    _verbose: bool,
) -> Result<CtagResult, Box<dyn Error>> {
    let client = create_confluence_client()?;
    let pages = client.query_pages_by_cql(&cql)?;

    // Parse colon-separated tag updates
    let updates: Result<Vec<(String, String)>, Box<dyn Error>> = tags
        .split(',')
        .map(|s| s.trim())
        .map(|update| -> Result<(String, String), Box<dyn Error>> {
            let parts: Vec<&str> = update.split(':').collect();
            if parts.len() != 2 {
                return Err(format!(
                    "Invalid update format '{}'. Expected 'old:new'",
                    update
                )
                .into());
            }
            Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
        })
        .collect();

    let updates = updates?;

    if pages.is_empty() {
        return Ok(CtagResult {
            success: true,
            operation: "update".to_string(),
            cql: cql.clone(),
            pages: Vec::new(),
            error: None,
            summary: format!("No pages found matching CQL: {}", cql),
        });
    }

    let page_results: Vec<CtagPageResult> = pages.iter().map(|page| {
        let labels = get_page_labels(page);
        CtagPageResult {
            id: page.id.clone(),
            title: page.title.clone(),
            labels,
            highlighted: false,
            children: Vec::new(),
        }
    }).collect();

    if !dry_run {
        let page_ids: Vec<&str> = pages.iter().map(|p| p.id.as_str()).collect();
        client.bulk_update_labels(&page_ids, &updates)?;
    }

    let summary = if dry_run {
        format!("DRY RUN: Would update labels {:?} on {} pages", updates, pages.len())
    } else {
        format!("Successfully updated labels {:?} on {} pages", updates, pages.len())
    };

    Ok(CtagResult {
        success: true,
        operation: "update".to_string(),
        cql: cql.clone(),
        pages: page_results,
        error: None,
        summary,
    })
}

/// Execute remove operation
fn execute_remove_operation(
    cql: String,
    tags: String,
    dry_run: bool,
    _verbose: bool,
) -> Result<CtagResult, Box<dyn Error>> {
    let client = create_confluence_client()?;
    let pages = client.query_pages_by_cql(&cql)?;

    let tag_list: Vec<&str> = tags.split(',').map(|s| s.trim()).collect();

    if pages.is_empty() {
        return Ok(CtagResult {
            success: true,
            operation: "remove".to_string(),
            cql: cql.clone(),
            pages: Vec::new(),
            error: None,
            summary: format!("No pages found matching CQL: {}", cql),
        });
    }

    let page_results: Vec<CtagPageResult> = pages.iter().map(|page| {
        let labels = get_page_labels(page);
        CtagPageResult {
            id: page.id.clone(),
            title: page.title.clone(),
            labels,
            highlighted: false,
            children: Vec::new(),
        }
    }).collect();

    if !dry_run {
        let page_ids: Vec<&str> = pages.iter().map(|p| p.id.as_str()).collect();
        client.bulk_remove_labels(&page_ids, &tag_list)?;
    }

    let summary = if dry_run {
        format!("DRY RUN: Would remove labels {:?} from {} pages", tag_list, pages.len())
    } else {
        format!("Successfully removed labels {:?} from {} pages", tag_list, pages.len())
    };

    Ok(CtagResult {
        success: true,
        operation: "remove".to_string(),
        cql: cql.clone(),
        pages: page_results,
        error: None,
        summary,
    })
}

/// Create a Confluence client using environment variables
fn create_confluence_client() -> Result<ConfluenceClient, Box<dyn Error>> {
    dotenv::dotenv().ok(); // Load .env file, ignore if not found

    let base_url =
        std::env::var("ATLASSIAN_URL").map_err(|_| "ATLASSIAN_URL environment variable not set")?;
    let username = std::env::var("ATLASSIAN_USERNAME")
        .map_err(|_| "ATLASSIAN_USERNAME environment variable not set")?;
    let api_token = std::env::var("ATLASSIAN_API_TOKEN")
        .map_err(|_| "ATLASSIAN_API_TOKEN environment variable not set")?;

    let config = ConfluenceConfig {
        base_url,
        username,
        api_token,
    };

    ConfluenceClient::new(config).map_err(|e| e.into())
}

/// Build tree structure from flat list of pages using ancestor relationships
fn build_tree_from_pages(
    page_results: Vec<CtagPageResult>,
    confluence_pages: &[ConfluencePage],
) -> Vec<CtagPageResult> {
    // Create a map from page ID to the original ConfluencePage for ancestor lookup
    let page_lookup: std::collections::HashMap<String, &ConfluencePage> =
        confluence_pages.iter().map(|p| (p.id.clone(), p)).collect();

    // Create a map from page ID to CtagPageResult for easy access
    let mut result_lookup: std::collections::HashMap<String, CtagPageResult> =
        page_results.into_iter().map(|p| (p.id.clone(), p)).collect();

    // Find root pages (pages with no parents in our result set)
    let mut roots = Vec::new();

    // Collect page IDs to avoid borrowing issues
    let page_ids: Vec<String> = result_lookup.keys().cloned().collect();

    for page_id in page_ids {
        // Check if this page has a parent in our result set
        let has_parent_in_results = if let Some(confluence_page) = page_lookup.get(&page_id) {
            confluence_page.ancestors
                .as_ref()
                .map(|ancestors| {
                    ancestors.iter().any(|ancestor| result_lookup.contains_key(&ancestor.id))
                })
                .unwrap_or(false)
        } else {
            false
        };

        if !has_parent_in_results {
            // This is a root page - move it from the lookup to roots
            if let Some(mut root) = result_lookup.remove(&page_id) {
                build_children_recursive(&mut root, &mut result_lookup, &page_lookup);
                roots.push(root);
            }
        }
    }

    // Add any remaining pages as additional roots (orphaned pages)
    for (_, page) in result_lookup {
        roots.push(page);
    }

    roots
}

/// Recursively build children for a page
fn build_children_recursive(
    parent: &mut CtagPageResult,
    result_lookup: &mut std::collections::HashMap<String, CtagPageResult>,
    page_lookup: &std::collections::HashMap<String, &ConfluencePage>,
) {
    let parent_id = parent.id.clone();
    let mut children_to_remove = Vec::new();

    // Find children of this parent
    for (child_id, _child_result) in result_lookup.iter() {
        if let Some(confluence_page) = page_lookup.get(child_id) {
            // Check if this child has the current parent in its ancestors
            let is_direct_child = confluence_page.ancestors
                .as_ref()
                .map(|ancestors| {
                    // Find the immediate parent (last ancestor)
                    ancestors.last()
                        .map(|last_ancestor| last_ancestor.id == parent_id)
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if is_direct_child {
                children_to_remove.push(child_id.clone());
            }
        }
    }

    // Move children from lookup to parent
    for child_id in children_to_remove {
        if let Some(mut child) = result_lookup.remove(&child_id) {
            // Recursively build this child's children
            build_children_recursive(&mut child, result_lookup, page_lookup);
            parent.children.push(child);
        }
    }
}

/// Extract labels from a page's metadata
fn get_page_labels(page: &ConfluencePage) -> Vec<String> {
    page.metadata
        .as_ref()
        .and_then(|m| m.labels.as_ref())
        .map(|labels| labels.results.iter().map(|l| l.name.clone()).collect())
        .unwrap_or_default()
}

/// Check if a page should be highlighted based on its labels
fn should_highlight_page(page_labels: &[String], highlight_tags: Option<&[&str]>) -> bool {
    if let Some(tags) = highlight_tags {
        page_labels
            .iter()
            .any(|label| tags.contains(&label.as_str()))
    } else {
        false
    }
}

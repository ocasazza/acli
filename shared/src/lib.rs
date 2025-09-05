use serde::{Deserialize, Serialize};

pub mod confluence;
pub mod errors;
pub mod models;

pub use confluence::*;
pub use errors::*;
pub use models::*;

/// A page with additional metadata information about actions to take.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ConfluencePageTree {
    /// The root page URL
    pub root_page: String,

    /// The current labels on the page.
    pub current_page_labels: Vec<String>,

    /// The label actions that should take place.
    pub tag_actions: Vec<PageLabelAction>,
}

/// Actions that can be executed against page labels.
///
/// Each variant holds the data required to perform that operation. The enum is
/// serialized with an externally tagged representation like:
/// { "action": "add", "tag": "example" }
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum PageLabelAction {
    /// Add a label to the page.
    Add { tag: String },
    /// Update an existing label (named `from`) to a new label (`to`).
    Update { from: String, to: String },
    /// Remove a label from the page.
    Delete { tag: String },
    /// List labels (no additional data).
    List,
}

impl ConfluencePageTree {
    /// Convenience constructor.
    pub fn new(root_page: impl Into<String>) -> Self {
        ConfluencePageTree {
            root_page: root_page.into(),
            current_page_labels: Vec::new(),
            tag_actions: Vec::new(),
        }
    }

    /// Return a copy of the current labels known for the page.
    ///
    /// Stubbed implementation: returns the in-memory labels vector.
    pub fn list_labels(&self) -> Vec<String> {
        self.current_page_labels.clone()
    }

    /// Stubbed: add a label locally and record the intended action.
    ///
    /// In a full implementation this would call the Confluence API.
    pub fn add_label(&mut self, tag: impl Into<String>) {
        let tag_s = tag.into();
        self.current_page_labels.push(tag_s.clone());
        self.tag_actions.push(PageLabelAction::Add { tag: tag_s });
    }

    /// Stubbed: update a label locally (replace occurrences) and record the action.
    ///
    /// In a full implementation this would call the Confluence API to rename the label.
    pub fn update_label(&mut self, from: &str, to: &str) {
        for lbl in &mut self.current_page_labels {
            if lbl == from {
                *lbl = to.to_string();
            }
        }
        self.tag_actions.push(PageLabelAction::Update {
            from: from.to_string(),
            to: to.to_string(),
        });
    }

    /// Stubbed: remove a label locally and record the action.
    ///
    /// In a full implementation this would call the Confluence API.
    pub fn delete_label(&mut self, tag: &str) {
        self.current_page_labels.retain(|lbl| lbl.as_str() != tag);
        self.tag_actions.push(PageLabelAction::Delete {
            tag: tag.to_string(),
        });
    }

    /// Stubbed: apply all recorded actions.
    ///
    /// This is a no-op placeholder that demonstrates the API surface for the
    /// shared library. If `dry_run` is true, the function will not perform
    /// destructive operations (still a no-op here) and will return Ok.
    pub fn apply_actions(
        &self,
        dry_run: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if dry_run {
            // In a real implementation, we'd log what we'd do.
            // Keep as a no-op for stubbing purposes.
            return Ok(());
        }

        // Placeholder: pretend we applied actions successfully.
        Ok(())
    }
}

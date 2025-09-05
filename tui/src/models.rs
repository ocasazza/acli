//! Data models for the hierarchical TUI navigation

use serde::{Deserialize, Serialize};

/// Represents an Atlassian domain/environment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtlassianDomain {
    /// Domain name/URL
    pub name: String,
    /// Base URL for the domain
    pub base_url: String,
    /// Available products in this domain
    pub products: Vec<AtlassianProduct>,
}

/// Represents an Atlassian product (Confluence, Jira, JSM)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtlassianProduct {
    /// Product type
    pub product_type: ProductType,
    /// Product name for display
    pub name: String,
    /// Available projects/spaces in this product
    pub projects: Vec<Project>,
    /// Whether this product is available/accessible
    pub available: bool,
}

/// Types of Atlassian products
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductType {
    Confluence,
    Jira,
    Jsm,
}

impl ProductType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ProductType::Confluence => "Confluence",
            ProductType::Jira => "Jira",
            ProductType::Jsm => "Jira Service Management",
        }
    }

    pub fn api_path(&self) -> &'static str {
        match self {
            ProductType::Confluence => "/wiki/rest/api",
            ProductType::Jira => "/rest/api/2",
            ProductType::Jsm => "/rest/servicedeskapi",
        }
    }
}

/// Represents a project or space within a product
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Project/space ID
    pub id: String,
    /// Project/space name
    pub name: String,
    /// Project/space key
    pub key: String,
    /// Project description
    pub description: Option<String>,
    /// Project type (for different project contexts)
    pub project_type: String,
}

/// Current navigation context in the TUI
#[derive(Debug, Clone)]
pub struct NavigationContext {
    /// Currently selected domain
    pub domain: Option<AtlassianDomain>,
    /// Currently selected product
    pub product: Option<AtlassianProduct>,
    /// Currently selected project/space
    pub project: Option<Project>,
}

impl Default for NavigationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationContext {
    pub fn new() -> Self {
        Self {
            domain: None,
            product: None,
            project: None,
        }
    }

    /// Check if we have a complete context (domain + product + project)
    pub fn is_complete(&self) -> bool {
        self.domain.is_some() && self.product.is_some() && self.project.is_some()
    }

    /// Get a display string for the current context
    pub fn display_path(&self) -> String {
        let mut parts = Vec::new();

        if let Some(domain) = &self.domain {
            parts.push(domain.name.clone());
        }

        if let Some(product) = &self.product {
            parts.push(product.name.clone());
        }

        if let Some(project) = &self.project {
            parts.push(project.name.clone());
        }

        if parts.is_empty() {
            "No selection".to_string()
        } else {
            parts.join(" > ")
        }
    }

    /// Generate a CQL context prefix for commands
    pub fn cql_context(&self) -> Option<String> {
        if let (Some(_), Some(product), Some(project)) = (&self.domain, &self.product, &self.project) {
            match product.product_type {
                ProductType::Confluence => Some(format!("space = \"{}\"", project.key)),
                ProductType::Jira => Some(format!("project = \"{}\"", project.key)),
                ProductType::Jsm => Some(format!("project = \"{}\"", project.key)),
            }
        } else {
            None
        }
    }
}

/// Tree node for navigation display
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Display name for this node
    pub name: String,
    /// Node type for different handling
    pub node_type: TreeNodeType,
    /// Whether this node is expanded
    pub expanded: bool,
    /// Child nodes
    pub children: Vec<TreeNode>,
    /// Whether this node is currently selected
    pub selected: bool,
}

/// Types of tree nodes
#[derive(Debug, Clone, PartialEq)]
pub enum TreeNodeType {
    Domain(AtlassianDomain),
    Product(AtlassianProduct),
    Project(Project),
}

impl TreeNode {
    pub fn new_domain(domain: AtlassianDomain) -> Self {
        let name = domain.name.clone();
        Self {
            name,
            node_type: TreeNodeType::Domain(domain),
            expanded: false,
            children: Vec::new(),
            selected: false,
        }
    }

    pub fn new_product(product: AtlassianProduct) -> Self {
        let name = product.name.clone();
        Self {
            name,
            node_type: TreeNodeType::Product(product),
            expanded: false,
            children: Vec::new(),
            selected: false,
        }
    }

    pub fn new_project(project: Project) -> Self {
        let name = project.name.clone();
        Self {
            name,
            node_type: TreeNodeType::Project(project),
            expanded: false,
            children: Vec::new(),
            selected: false,
        }
    }

    /// Toggle expansion state
    pub fn toggle_expansion(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Set selection state
    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

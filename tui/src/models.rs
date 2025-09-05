//! Data models for the TUI application

// Re-export shared models
pub use nix_rust_template::{AtlassianDomain, AtlassianProduct, ProductType, Project};

/// Tree node for navigation display
#[derive(Clone, Debug)]
pub struct TreeNode {
    /// Display name
    pub name: String,
    /// Node type and associated data
    pub node_type: TreeNodeType,
    /// Whether this node is expanded
    pub expanded: bool,
    /// Whether this node is selected
    pub selected: bool,
    /// Child nodes
    pub children: Vec<TreeNode>,
}

/// Type alias for complex tree item tuple used in search and display
/// Format: (name, depth, selected, score, match_positions, original_index)
pub type TreeItemWithMetadata = (String, usize, bool, isize, Vec<usize>, usize);

/// Type alias for simple tree item tuple
/// Format: (name, depth, selected)
pub type TreeItem = (String, usize, bool);

/// Types of tree nodes
#[derive(Clone, Debug)]
pub enum TreeNodeType {
    /// Domain node
    Domain(AtlassianDomain),
    /// Product node
    Product(AtlassianProduct),
    /// Project/Space node
    Project(Project),
}

impl TreeNode {
    /// Create a new domain node
    pub fn new_domain(domain: AtlassianDomain) -> Self {
        Self {
            name: domain.name.clone(),
            node_type: TreeNodeType::Domain(domain),
            expanded: true, // Domains start expanded
            selected: false,
            children: Vec::new(),
        }
    }

    /// Create a new product node
    pub fn new_product(product: AtlassianProduct) -> Self {
        Self {
            name: product.name.clone(),
            node_type: TreeNodeType::Product(product),
            expanded: false,
            selected: false,
            children: Vec::new(),
        }
    }

    /// Create a new project node
    pub fn new_project(project: Project) -> Self {
        Self {
            name: project.name.clone(),
            node_type: TreeNodeType::Project(project),
            expanded: false,
            selected: false,
            children: Vec::new(),
        }
    }
}

/// Navigation context for hierarchical selection
#[derive(Clone, Debug, Default)]
pub struct NavigationContext {
    /// Selected domain
    pub domain: Option<AtlassianDomain>,
    /// Selected product
    pub product: Option<AtlassianProduct>,
    /// Selected project/space
    pub project: Option<Project>,
}

impl NavigationContext {
    /// Create a new empty navigation context
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if we have a complete context for command execution
    pub fn is_complete(&self) -> bool {
        self.domain.is_some() && self.product.is_some() && self.project.is_some()
    }

    /// Display the current navigation path
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

    /// Get CQL context for command execution
    pub fn cql_context(&self) -> Option<String> {
        if !self.is_complete() {
            return None;
        }

        if let (Some(_domain), Some(product), Some(project)) =
            (&self.domain, &self.product, &self.project)
        {
            // Create CQL context based on product type and project
            match product.product_type {
                ProductType::Confluence => {
                    // For Confluence, use space key
                    Some(format!("space = '{}'", project.key))
                }
                ProductType::Jira => {
                    // For Jira, use project key
                    Some(format!("project = '{}'", project.key))
                }
                ProductType::Jsm => {
                    // For JSM, also use project key but might need different handling
                    Some(format!("project = '{}'", project.key))
                }
            }
        } else {
            None
        }
    }
}

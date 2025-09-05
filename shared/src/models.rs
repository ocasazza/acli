//! Domain models for Atlassian services

/// Product types available in Atlassian
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductType {
    /// Confluence wiki/knowledge base
    Confluence,
    /// Jira issue tracking
    Jira,
    /// Jira Service Management
    Jsm,
}

/// Represents an Atlassian domain/organization
#[derive(Clone, Debug)]
pub struct AtlassianDomain {
    /// Domain name
    pub name: String,
    /// Base URL for the domain
    pub base_url: String,
    /// Available products in this domain
    pub products: Vec<AtlassianProduct>,
}

/// Represents an Atlassian product (Confluence, Jira, etc.)
#[derive(Clone, Debug)]
pub struct AtlassianProduct {
    /// Product type
    pub product_type: ProductType,
    /// Product name
    pub name: String,
    /// Projects/spaces within this product
    pub projects: Vec<Project>,
    /// Whether this product is available/accessible
    pub available: bool,
}

/// Represents a project or space within a product
#[derive(Clone, Debug)]
pub struct Project {
    /// Project/space ID
    pub id: String,
    /// Project/space name
    pub name: String,
    /// Project/space key
    pub key: String,
    /// Project/space description
    pub description: Option<String>,
    /// Project type (e.g., "space", "project")
    pub project_type: String,
}

//! Domain loading and Atlassian service discovery

use crate::models::{AtlassianDomain, AtlassianProduct, ProductType, Project};
use nix_rust_template::ConfluenceClient;
use std::error::Error;

/// Domain loader for discovering and loading Atlassian services
pub struct DomainLoader {
    confluence_client: ConfluenceClient,
}

impl DomainLoader {
    /// Create a new domain loader with the given Confluence client
    pub fn new(confluence_client: ConfluenceClient) -> Self {
        Self { confluence_client }
    }

    /// Load domain data from environment variables and discover products/projects
    pub fn load_domain_data(&self) -> Result<AtlassianDomain, Box<dyn Error>> {
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
        let confluence_product = self.discover_confluence_product()?;
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

        Ok(domain)
    }

    /// Discover Confluence product and its spaces
    fn discover_confluence_product(&self) -> Result<AtlassianProduct, Box<dyn Error>> {
        match self.confluence_client.get_spaces() {
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

                Ok(AtlassianProduct {
                    product_type: ProductType::Confluence,
                    name: "Confluence".to_string(),
                    projects: confluence_projects,
                    available: true,
                })
            }
            Err(e) => {
                // Log the actual error for debugging
                eprintln!("Confluence API error: {e:?}");
                Ok(AtlassianProduct {
                    product_type: ProductType::Confluence,
                    name: format!("Confluence (Error: {e})"),
                    projects: Vec::new(),
                    available: false,
                })
            }
        }
    }
}

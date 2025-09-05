use crate::errors::{ConfluenceError, Result};
use base64::Engine;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use url::Url;

/// Configuration for connecting to a Confluence instance.
#[derive(Debug, Clone)]
pub struct ConfluenceConfig {
     /// Base URL of the Confluence instance (e.g., "<https://company.atlassian.net>")
    pub base_url: String,
    /// API token for authentication
    pub api_token: String,
    /// Username/email for authentication
    pub username: String,
}

/// Represents a Confluence page returned from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfluencePage {
    /// Page ID
    pub id: String,
    /// Page title
    pub title: String,
    /// Page type (usually "page")
    #[serde(rename = "type")]
    pub page_type: String,
    /// Page status (usually "current")
    pub status: String,
    /// Page URL links
    #[serde(rename = "_links")]
    pub links: Option<PageLinks>,
    /// Page ancestors (parent pages)
    pub ancestors: Option<Vec<ConfluencePage>>,
    /// Page labels
    pub metadata: Option<PageMetadata>,
}

/// Links associated with a Confluence page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLinks {
    /// Web UI link
    pub webui: Option<String>,
    /// API self link
    #[serde(rename = "self")]
    pub self_link: Option<String>,
}

/// Metadata for a Confluence page, including labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    /// Page labels
    pub labels: Option<PageLabels>,
}

/// Container for page labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLabels {
    /// Array of label results
    pub results: Vec<PageLabel>,
    /// Total number of labels
    pub size: Option<i32>,
}

/// Individual page label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLabel {
    /// Label ID
    pub id: Option<String>,
    /// Label name
    pub name: String,
    /// Label prefix (usually "global")
    pub prefix: Option<String>,
}

/// Response from CQL search queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CqlSearchResponse {
    /// Search results
    pub results: Vec<ConfluencePage>,
    /// Start index for pagination
    pub start: i32,
    /// Limit for pagination
    pub limit: i32,
    /// Total number of results
    pub size: i32,
}

/// Request body for adding labels to a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddLabelsRequest {
    /// Array of labels to add
    pub labels: Vec<LabelRequest>,
}

/// Individual label in a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelRequest {
    /// Label prefix (usually "global")
    pub prefix: String,
    /// Label name
    pub name: String,
}

/// Represents a Confluence space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfluenceSpace {
    /// Space ID (can be integer or string from API)
    #[serde(deserialize_with = "deserialize_id_as_string")]
    pub id: String,
    /// Space key
    pub key: String,
    /// Space name
    pub name: String,
    /// Space type (usually "global")
    #[serde(rename = "type")]
    pub space_type: String,
    /// Space status (usually "current")
    pub status: String,
    /// Space description
    pub description: Option<SpaceDescription>,
    /// Space links
    #[serde(rename = "_links")]
    pub links: Option<SpaceLinks>,
}

/// Custom deserializer to handle both integer and string IDs
fn deserialize_id_as_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct IdVisitor;

    impl<'de> Visitor<'de> for IdVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or integer")
        }

        fn visit_str<E>(self, value: &str) -> std::result::Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_u64<E>(self, value: u64) -> std::result::Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_i64<E>(self, value: i64) -> std::result::Result<String, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(IdVisitor)
}

/// Space description content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceDescription {
    /// Plain text description
    pub plain: Option<SpacePlainDescription>,
}

/// Plain text space description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacePlainDescription {
    /// Description value
    pub value: String,
}

/// Links associated with a Confluence space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceLinks {
    /// Web UI link
    pub webui: Option<String>,
    /// API self link
    #[serde(rename = "self")]
    pub self_link: Option<String>,
}

/// Response from spaces API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacesResponse {
    /// Array of spaces
    pub results: Vec<ConfluenceSpace>,
    /// Start index for pagination
    pub start: i32,
    /// Limit for pagination
    pub limit: i32,
    /// Total number of spaces
    pub size: i32,
}

/// Client for interacting with the Confluence REST API.
pub struct ConfluenceClient {
    client: Client,
    config: ConfluenceConfig,
    headers: HeaderMap,
}

impl ConfluenceClient {
    /// Create a new Confluence client with the given configuration.
    pub fn new(config: ConfluenceConfig) -> Result<Self> {
        // Validate base URL
        let _base_url = Url::parse(&config.base_url).map_err(|_| ConfluenceError::Config {
            message: format!("Invalid base URL: {}", config.base_url),
        })?;
        // Set up HTTP client
        let client = Client::new();
        // Set up authentication headers
        let mut headers = HeaderMap::new();
        // Use basic auth with username and API token
        let auth_string = format!("{}:{}", config.username, config.api_token);
        let auth_header = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(&auth_string)
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_header).map_err(|_| ConfluenceError::Authentication {
                message: "Failed to create authorization header".to_string(),
            })?,
        );

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(ConfluenceClient {
            client,
            config,
            headers,
        })
    }

    /// Execute a CQL query and return matching pages.
    pub fn query_pages_by_cql(&self, cql: &str) -> Result<Vec<ConfluencePage>> {
        let url = format!(
            "{}/wiki/rest/api/content/search?cql={}&expand=metadata.labels,ancestors",
            self.config.base_url,
            urlencoding::encode(cql)
        );

        let response = self.client.get(&url).headers(self.headers.clone()).send()?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ConfluenceError::CqlQuery {
                query: cql.to_string(),
                message: format!("HTTP {status}: {error_text}"),
            });
        }

        let search_response: CqlSearchResponse = response.json()?;
        Ok(search_response.results)
    }

    /// Get labels for a specific page.
    pub fn get_page_labels(&self, page_id: &str) -> Result<Vec<String>> {
        let url = format!(
            "{}/wiki/rest/api/content/{}/label",
            self.config.base_url, page_id
        );

        let response = self.client.get(&url).headers(self.headers.clone()).send()?;

        if response.status() == 404 {
            return Err(ConfluenceError::PageNotFound {
                page_id: page_id.to_string(),
            });
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ConfluenceError::ApiError {
                status,
                message: error_text,
            });
        }

        let labels_response: PageLabels = response.json()?;
        Ok(labels_response
            .results
            .into_iter()
            .map(|l| l.name)
            .collect())
    }

    /// Add labels to a page.
    pub fn add_page_labels(&self, page_id: &str, labels: &[&str]) -> Result<()> {
        let url = format!(
            "{}/wiki/rest/api/content/{}/label",
            self.config.base_url, page_id
        );

        let request_body = AddLabelsRequest {
            labels: labels
                .iter()
                .map(|name| LabelRequest {
                    prefix: "global".to_string(),
                    name: name.to_string(),
                })
                .collect(),
        };

        let response = self
            .client
            .post(&url)
            .headers(self.headers.clone())
            .json(&request_body)
            .send()?;

        if response.status() == 404 {
            return Err(ConfluenceError::PageNotFound {
                page_id: page_id.to_string(),
            });
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ConfluenceError::LabelOperation {
                message: format!(
                    "Failed to add labels to page {page_id}: HTTP {status}: {error_text}"
                ),
            });
        }

        Ok(())
    }

    /// Remove labels from a page.
    pub fn remove_page_labels(&self, page_id: &str, labels: &[&str]) -> Result<()> {
        for label in labels {
            let url = format!(
                "{}/wiki/rest/api/content/{}/label/{}",
                self.config.base_url,
                page_id,
                urlencoding::encode(label)
            );

            let response = self
                .client
                .delete(&url)
                .headers(self.headers.clone())
                .send()?;

            if response.status() == 404 {
                // Label might not exist, which is okay for removal
                continue;
            }

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let error_text = response
                    .text()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(ConfluenceError::LabelOperation {
                    message: format!("Failed to remove label '{label}' from page {page_id}: HTTP {status}: {error_text}"),
                });
            }
        }

        Ok(())
    }

    /// Update a label on a page (remove old, add new).
    pub fn update_page_label(&self, page_id: &str, old_label: &str, new_label: &str) -> Result<()> {
        // Remove the old label
        self.remove_page_labels(page_id, &[old_label])?;

        // Add the new label
        self.add_page_labels(page_id, &[new_label])?;

        Ok(())
    }

    /// Apply bulk operations to multiple pages.
    pub fn bulk_add_labels(&self, page_ids: &[&str], labels: &[&str]) -> Result<()> {
        for page_id in page_ids {
            self.add_page_labels(page_id, labels)?;
        }
        Ok(())
    }

    /// Bulk remove labels from multiple pages.
    pub fn bulk_remove_labels(&self, page_ids: &[&str], labels: &[&str]) -> Result<()> {
        for page_id in page_ids {
            self.remove_page_labels(page_id, labels)?;
        }
        Ok(())
    }

    /// Bulk update labels on multiple pages.
    pub fn bulk_update_labels(
        &self,
        page_ids: &[&str],
        updates: &[(String, String)],
    ) -> Result<()> {
        for page_id in page_ids {
            for (old_label, new_label) in updates {
                self.update_page_label(page_id, old_label, new_label)?;
            }
        }
        Ok(())
    }

    /// Get all spaces in the Confluence instance.
    pub fn get_spaces(&self) -> Result<Vec<ConfluenceSpace>> {
        let url = format!(
            "{}/wiki/rest/api/space?expand=description.plain&limit=1000",
            self.config.base_url
        );

        let response = self.client.get(&url).headers(self.headers.clone()).send()?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ConfluenceError::ApiError {
                status,
                message: format!("Failed to get spaces: HTTP {status}: {error_text}"),
            });
        }

        let spaces_response: SpacesResponse = response.json()?;
        Ok(spaces_response.results)
    }

    /// Check if Confluence API is accessible.
    pub fn check_connectivity(&self) -> Result<bool> {
        let url = format!("{}/wiki/rest/api/space", self.config.base_url);

        let response = self.client.head(&url).headers(self.headers.clone()).send()?;

        Ok(response.status().is_success())
    }
}

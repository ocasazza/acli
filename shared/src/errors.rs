use thiserror::Error;

/// Errors that can occur when interacting with the Confluence API.
#[derive(Error, Debug)]
pub enum ConfluenceError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization failed
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Authentication failed
    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    /// Page not found
    #[error("Page not found: {page_id}")]
    PageNotFound { page_id: String },

    /// Label operation failed
    #[error("Label operation failed: {message}")]
    LabelOperation { message: String },

    /// CQL query error
    #[error("CQL query failed: {query} - {message}")]
    CqlQuery { query: String, message: String },

    /// API returned an error response
    #[error("API error {status}: {message}")]
    ApiError { status: u16, message: String },

    /// Invalid URL provided
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// Configuration error
    #[error("Configuration error: {message}")]
    Config { message: String },
}

pub type Result<T> = std::result::Result<T, ConfluenceError>;

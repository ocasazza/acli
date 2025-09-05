//! ACLI TUI - Terminal User Interface for Atlassian CLI
//!
//! This crate provides an interactive TUI for working with Atlassian products.

use nix_rust_template::{ConfluenceClient, ConfluenceConfig};
use std::error::Error;

pub mod app;
pub mod command;
pub mod domain_loader;
pub mod event;
pub mod event_handler;
pub mod models;
pub mod screens;
pub mod search;
pub mod signal_handler;
pub mod terminal_manager;
pub mod tree_navigation;
pub mod ui;

pub use app::App;

/// Main entry point for the TUI application
pub fn run_tui() -> Result<(), Box<dyn Error>> {
    let app = App::new()?;
    app.run()
}

/// Create a Confluence client using environment variables
pub fn create_confluence_client() -> Result<ConfluenceClient, Box<dyn Error>> {
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

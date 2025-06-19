use crate::common::dynamic_proxy;
use crate::db::db::{connect_db, initialize_schema};
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;

/// Sets up the working directory and root data directory
pub fn setup_directories() -> Result<PathBuf> {
    println!("[SETUP] Setting up directories...");

    // Setup Working Directory and Root Data Directory
    let current_dir =
        current_dir().with_context(|| "Failed to get the current working directory.")?;
    println!("[MAIN] Current working directory: {:?}", current_dir);

    let root_data_dir = PathBuf::from("downloaded_manhwa"); // Main folder to store downloaded manhwa
    if !root_data_dir.exists() {
        fs::create_dir_all(&root_data_dir).with_context(|| {
            format!(
                "Failed to create root data directory: {}",
                root_data_dir.display()
            )
        })?;
        println!("[MAIN] Root data directory created: {:?}", root_data_dir);
    }

    Ok(root_data_dir)
}

/// Sets up the database connection and initializes the schema
pub fn setup_database(db_path: &str, schema_path: &str) -> Result<Connection> {
    println!("[MAIN]  Connecting to database: {}", db_path);
    let conn = connect_db(db_path)
        .with_context(|| format!("Failed to connect to database: {}", db_path))?;

    initialize_schema(&conn, schema_path)
        .with_context(|| format!("Failed to initialize database schema from {}", schema_path))?;
    println!("[MAIN] Database and schema initialized successfully.");

    Ok(conn)
}

/// Sets up the HTTP client
pub fn setup_http_client() -> Result<reqwest::Client> {
    println!("[SETUP] Initializing HTTP client...");

    // Initialize HTTP Client
    // This client will be used for all HTTP requests.
    // Note: fetcher::fetch_html creates its own client. This should be consolidated.
    let client = dynamic_proxy::init_client()
        .context("Failed to initialize HTTP client from dynamic_proxy")?;
    println!("[MAIN] HTTP Client created successfully.");

    Ok(client)
}

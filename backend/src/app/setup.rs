use anyhow::{Context, Result};
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;

use crate::common::dynamic_proxy;
use crate::db::db::{DatabaseService, create_db_pool, initialize_schema};

/// Sets up the database connection and initializes the schema
pub fn setup_database(db_path: &str, schema_path: &str) -> Result<DatabaseService> {
    println!("[MAIN]  Connecting to database: {}", db_path);

    // Create connection pool
    let pool = create_db_pool(db_path)
        .with_context(|| format!("Failed to create database pool for: {}", db_path))?;

    // Get a single connection to initialize the schema
    let mut conn = pool
        .get()
        .context("Failed to get a connection from the pool for schema initialization")?;

    initialize_schema(&mut conn, schema_path)
        .with_context(|| format!("Failed to initialize database schema from {}", schema_path))?;
    println!("[MAIN] Database and schema initialized successfully.");

    // Create and return the Database Service
    let db_service = DatabaseService::new(pool);
    Ok(db_service)
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

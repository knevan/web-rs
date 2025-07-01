use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;

use crate::common::dynamic_proxy;
use crate::db::db::DatabaseService;

/// Sets up the database connection pool for PostgreSQL using sqlx.
/// Schema initialization is now handled by `sqlx-cli migrate`.
pub async fn setup_database() -> Result<DatabaseService> {
    // Get database URL from environment variables
    let db_url = env::var("DATABASE_URL")
        .context("[SETUP] Database URL not found in environment variables")?;

    println!("[MAIN]  Connecting to postgres database via sqlx");

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .min_connections(2)
        .max_lifetime(Duration::from_secs(300))
        .idle_timeout(Duration::from_secs(60))
        .test_before_acquire(true)
        .connect(&db_url)
        .await
        .context("[SETUP] Failed to create sqlx Postgres connection pool")?;

    println!("[MAIN] Sqlx connection pool created successfully.");
    println!("[MAIN] Remember to run `sqlx migrate run` to apply schema changes.");

    // Create and return the Database Service
    let db_service = DatabaseService::new(pool);
    Ok(db_service)
}

/// Sets up the HTTP client
pub fn setup_http_client() -> Result<reqwest::Client> {
    println!("[SETUP] Initializing HTTP client...");

    // Initialize HTTP Client
    // This client will be used for all HTTP requests.
    // [Note] fetcher::fetch_html creates its own client. This should be consolidated.
    let client = dynamic_proxy::init_client()
        .context("Failed to initialize HTTP client from dynamic_proxy")?;
    println!("[MAIN] HTTP Client created successfully.");

    Ok(client)
}

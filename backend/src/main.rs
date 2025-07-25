#![allow(clippy::uninlined_format_args)]
extern crate core;
mod app;
mod auth;
mod builder;
mod common;
mod db;
mod encoding;
mod scraping;

use crate::builder::startup;
use crate::builder::startup::Mailer;
use crate::common::dynamic_proxy;
use anyhow::{Context, Result};
use dotenvy::dotenv;
use lettre::transport::smtp::authentication::Credentials;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;

// Main entry point for the application
#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();
    println!("[MAIN] Environment variables loaded.");
    println!("[MAIN] App Starting...");

    // Initialize database external resources
    let db_url =
        env::var("DATABASE_URL").context("[MAIN] DATABASE_URL must be set")?;
    let db_pool = PgPoolOptions::new()
        .max_connections(4)
        .min_connections(2)
        .max_lifetime(Duration::from_secs(300))
        .idle_timeout(Duration::from_secs(60))
        .test_before_acquire(true)
        .connect(&db_url)
        .await
        .context("[MAIN] Failed to create sqlx Postgres connection pool")?;

    println!("[MAIN] Database pool created.");

    // Initialize Mailer service external resources
    let smtp_server =
        env::var("SMTP_SERVER").context("[MAIN] SMTP_SERVER must be set")?;
    let smtp_username = env::var("SMTP_USERNAME")
        .context("[MAIN] SMTP_USERNAME must be set")?;
    let smtp_password = env::var("SMTP_PASSWORD")
        .context("[MAIN] SMTP_PASSWORD must be set")?;

    let creds = Credentials::new(smtp_username, smtp_password);
    let mailer = Mailer::starttls_relay(&smtp_server)
        .context("[MAIN] Failed to build SMTP relay")?
        .credentials(creds)
        .build();

    println!("[MAIN] Mailer service initialized.");

    // Initialize HTTP Client
    let http_client = dynamic_proxy::init_client()
        .context("[MAIN] Failed to initialize HTTP client")?;
    println!("[MAIN] HTTP Client created.");

    // Define the server address, port and listeners
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("[MAIN] Server listening on https://{addr}");

    // Start the builder
    startup::run(listener, db_pool, mailer, http_client).await?;

    Ok(())
}

mod app;
mod auth;
mod common;
mod db;
mod encoding;
mod scraping;

use crate::app::management::{
    get_or_create_series, get_series_host, prepare_series_directory, update_series_after_scraping,
};
use crate::app::orchestrator::scrape_series;
use crate::app::{config, setup};
use anyhow::Result;
use axum::Router;

#[tokio::main]
async fn main() -> Result<()> {
    println!("[MAIN] App Starting...");

    // Load application configuration
    let app_config = config::load_app_config("backend/config.toml")?;

    // Setup directories, database, and HTTP client
    let root_data_dir = setup::setup_directories()?;
    let conn = setup::setup_database("manhwa_list.sqlite3", "backend/src/db/db.sql")?;
    let http_client = setup::setup_http_client()?;

    let app = Router::new().merge(auth::routes::routes());

    // Series configuration
    let target_series_title = "Limit-Breaking Genius Mage";
    let target_series_url_default = "https://www.mgeko.cc/manga/limit-breaking-genius-mage/";
    let default_check_interval: i32 = 120; // minutes
    let max_chapters_to_scrape = 2;

    // Get or create series in database
    let mut series_data = get_or_create_series(
        &conn,
        target_series_title,
        target_series_url_default,
        default_check_interval,
    )?;

    // Prepare directory for series
    let series_base_path = prepare_series_directory(&root_data_dir, &series_data)?;

    // Get site configuration for the series
    let series_host = get_series_host(&series_data)?;
    let site_config = app_config.get_site_config(&series_host)
                                .ok_or_else(|| anyhow::anyhow!(
            "No scraping configuration found for host: {}. Ensure the host is listed in config.toml.", 
            series_host
        ))?;

    println!(
        "[MAIN] Using scraping configuration for host: {}",
        site_config.host_name
    );

    // Execute scraping process
    match scrape_series(
        &series_data,
        &series_base_path,
        &http_client,
        site_config,
        max_chapters_to_scrape,
    )
    .await
    {
        Ok(last_downloaded_chapter) => {
            // Update series data in database
            update_series_after_scraping(&conn, &mut series_data, last_downloaded_chapter)?;
        }
        Err(e) => {
            eprintln!("[MAIN] Error during scraping process: {}", e);
            // Could update series status to 'error' here if needed
        }
    }

    println!("\n[MAIN] Scraper application finished.");
    Ok(())
}

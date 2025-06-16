#![warn(clippy::perf)]
mod core;
mod db;
mod encoding;
mod scraping;
mod user;

use crate::core::utils::{random_sleep_time, sanitize_series_title};
use crate::db::db::{
    add_manhwa_series, connect_db, current_timestamp, get_manhwa_series_by_id,
    get_manhwa_series_by_title, initialize_schema, update_series_check_schedule,
    update_series_last_local_chapter, update_series_source_urls,
};
use crate::scraping::model::AppConfig;
use crate::scraping::{coordinator, fetcher, parser};
use anyhow::{Context, Result};
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    println!("[MAIN] App Starting...");

    // Load Application Configuration from config.toml
    // This is the first crucial step to get scraping parameters.
    let app_config = AppConfig::load("backend/config.toml")
        .with_context(|| "Failed to load application configuration from config.toml. Ensure the file exists and its format is correct.")?;
    println!("[MAIN] Application configuration loaded successfully.");

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

    // Setup Database
    let db_path = "manhwa_list.sqlite3"; // Path to the SQLite database file
    let db_sql_file = "backend/src/db/db.sql"; // Path to the SQL schema file (if any)

    println!("[MAIN]  Connecting to database: {}", db_path);
    let conn = connect_db(db_path)
        .with_context(|| format!("Failed to connect to database: {}", db_path))?;

    initialize_schema(&conn, db_sql_file)
        .with_context(|| format!("Failed to initialize database schema from {}", db_sql_file))?;
    println!("[MAIN] Database and schema initialized successfully.");

    // Create AppState with database connection
    /*let async_conn = AsyncConnection::open(db_path).await?;

    let state = Arc::new(AppState {
        db: Arc::new(async_conn),
    });

    let app = create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;*/

    // Initialize HTTP Client
    // This client will be used for all HTTP requests.
    // Note: fetcher::fetch_html creates its own client. This should be consolidated.
    let http_client = core::dynamic_proxy::init_client()
        .context("Failed to initialize HTTP client from dynamic_proxy")?;
    println!("[MAIN] HTTP Client created successfully.");

    // Logic for Selecting Series to Process (Example: One Target Series for Testing)
    // TODO)): Make this configurable or dynamic instead of hardcoded.
    let target_series_title = "Limit-Breaking Genius Mage"; // Title series
    let target_series_url_default = "https://www.mgeko.cc/manga/limit-breaking-genius-mage/"; // Default URL series
    let default_check_interval: i32 = 120; // Default check interval in minutes

    // Get or Create Series Entry in Database
    let mut series_data = match get_manhwa_series_by_title(&conn, target_series_title)? {
        Some(mut series) => {
            println!(
                "[MAIN] Series '{}' found in database (ID: {}).",
                series.title, series.id
            );
            // Update source URL if different or empty in DB
            if series
                .current_source_url
                .clone()
                .is_none_or(|s| s.trim().is_empty() || s != target_series_url_default)
            {
                println!(
                    "[MAIN] Source URL for '{}' is different or empty in DB, updating with default URL: {}",
                    series.title, target_series_url_default
                );
                update_series_source_urls(&conn, series.id, target_series_url_default)?;
                // Update local series data after DB change
                series.current_source_url = Some(target_series_url_default.to_string());
                series.source_website_host = Url::parse(target_series_url_default)
                    .ok()
                    .and_then(|u| u.host_str().map(String::from));
            }
            series
        }
        None => {
            println!(
                "[MAIN] Series '{}' not found in database. Adding...",
                target_series_title
            );
            let new_series_id = add_manhwa_series(
                &conn,
                target_series_title,
                Some(target_series_url_default),
                default_check_interval,
            )?;
            println!(
                "[MAIN] Series '{}' added successfully with ID: {}.",
                target_series_title, new_series_id
            );
            // Fetch the newly added series data for consistency
            get_manhwa_series_by_id(&conn, new_series_id as i32)?
                .expect("Newly added series should exist.")
        }
    };

    // Prepare Local Folder Path for This Series (LOCAL TEST ONLY)
    let series_folder_name = sanitize_series_title(&series_data.title);
    let series_base_path = root_data_dir.join(&series_folder_name);

    if !series_base_path.exists() {
        fs::create_dir_all(&series_base_path)
            .with_context(|| format!("Failed to create series folder: {:?}", series_base_path))?;
        println!("[MAIN] Series folder created: {:?}", series_base_path);
    }

    // Get Specific Scraping Configuration for the Site from the Series URL
    let series_main_page_url = series_data.current_source_url.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "Series source URL does not exist for '{}'",
            series_data.title
        )
    })?;

    let series_host = Url::parse(series_main_page_url)?
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("Could not get host from URL: {}", series_main_page_url))?
        .to_string();

    // Get site configuration from the loaded AppConfig
    let site_config = app_config.get_site_config(&series_host)
                                .ok_or_else(|| anyhow::anyhow!("No scraping configuration found for host: {}. Ensure the host is listed in config.toml.", series_host))?;

    println!(
        "[MAIN] Using scraping configuration for host: {}",
        site_config.host_name
    );

    // Fetch and Parse Series Main Page to Get Chapter List
    println!(
        "[MAIN] Fetching series main page HTML from: {}",
        series_main_page_url
    );
    // Pass the shared http_client to fetch_html
    let series_page_html = fetcher::fetch_html(&http_client, series_main_page_url)
        .await
        .with_context(|| {
            format!(
                "Failed to fetch series main page HTML: {}",
                series_main_page_url
            )
        })?;

    // Short pause (sleep) after fetch to avoid rate limiting
    // tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    random_sleep_time(2, 6).await;

    // Extract all chapter links from the series main page using site configuration
    let all_available_chapters = parser::extract_chapter_links(
        &series_page_html,
        series_main_page_url, // Series page URL, to create absolute URLs if needed
        site_config,          // Pass reference to the relevant site configuration
    )
    .await
    .with_context(|| "Failed to extract chapter links from series page.")?;

    if all_available_chapters.is_empty() {
        println!(
            "[MAIN] No chapters found on the series page for '{}'. CSS selector might be wrong or no chapters available.",
            series_data.title
        );
    } else {
        println!(
            "[MAIN] Found {} chapters for series '{}'.",
            all_available_chapters.len(),
            series_data.title
        );

        // Logic to Determine Which Chapters to Scrape
        // Example: scrape the first 2 chapters newer than what's already present, or the first 2 if none exist.
        // TODO)): Make the number of chapters to scrape configurable.
        let last_local_chapter_num = series_data.last_chapter_found_locally.unwrap_or(0.0);
        let chapters_to_actually_scrape: Vec<parser::ChapterInfo> = all_available_chapters
            .into_iter()
            // Get newer chapters
            // .rev() // If you want from newest to oldest (might need to sort by number desc in parser first)
            .filter(|ch_info| ch_info.number > last_local_chapter_num)
            .take(2) // Take N chapters for testing (e.g., 2)
            .collect();

        if chapters_to_actually_scrape.is_empty() {
            println!(
                "[MAIN] No new chapters to scrape for series '{}' (last local: {}).",
                series_data.title, last_local_chapter_num
            );
        } else {
            println!(
                "[MAIN] Will scrape {} chapters for series '{}': {:?}",
                chapters_to_actually_scrape.len(),
                series_data.title,
                chapters_to_actually_scrape
                    .iter()
                    .map(|c| c.number)
                    .collect::<Vec<f32>>()
            );

            // Start Scraping Process for Selected Chapters
            match coordinator::process_series_chapters_from_list(
                &series_data,
                &series_base_path,
                &chapters_to_actually_scrape, // List of filtered chapters
                &http_client,                 // Pass the shared client
                site_config,                  // Pass site configuration to coordinator
            )
            .await
            {
                Ok(Some(last_downloaded_chapter_num_this_run)) => {
                    println!(
                        "[MAIN] Batch scraping finished. Last chapter downloaded this session: {}",
                        last_downloaded_chapter_num_this_run
                    );
                    // Update the last locally found chapter in DB if a new one was downloaded
                    if last_downloaded_chapter_num_this_run > last_local_chapter_num {
                        println!(
                            "[MAIN] Updating last_chapter_found_locally in DB for series '{}' to {}.",
                            series_data.title, last_downloaded_chapter_num_this_run
                        );
                        update_series_last_local_chapter(
                            &conn,
                            series_data.id,
                            Some(last_downloaded_chapter_num_this_run),
                        )?;
                        // Update series_data in memory as well
                        series_data.last_chapter_found_locally =
                            Some(last_downloaded_chapter_num_this_run);
                    }
                }
                Ok(None) => {
                    println!(
                        "[MAIN] Batch scraping finished. No new chapters were successfully downloaded this session."
                    );
                }
                Err(e) => {
                    eprintln!(
                        "[MAIN] An error occurred during the scraping process for series '{}': {}",
                        series_data.title, e
                    );
                    // Consider updating series status in DB to 'error' here
                    // db::update_series_processing_status(&conn, series_data.id, "error")?;
                }
            }
        }
    }

    // Update Check Schedule in Database
    // Mark that this series has been processed and when to check it again.
    let current_ts = current_timestamp();
    // TODO)): The next check time should be calculated based on series_data.check_interval_minutes
    // For now, just updating last_processed_timestamp and status.
    // The `next_check_timestamp` should ideally be `current_ts + interval`
    update_series_check_schedule(
        &conn,
        series_data.id,
        Some("monitoring"),
        Some(current_ts),
        None,
    )?;
    println!(
        "[MAIN] Check schedule for series '{}' has been updated.",
        series_data.title
    );

    println!("\n[MAIN] Scraper application finished.");
    Ok(())
}

/*let jpg_input_folder: &str = "imgjpg";
let jpg_output_folder: &str = "outputjpg";
let png_input_folder: &str = "imgpng";
let png_output_folder: &str = "outputpng";

println!("=== Program Konversi Gambar ke AVIF ===\n");

if !Path::new(jpg_input_folder).exists() {
    eprintln!(
        "Error: File input '{}' tidak ditemukan. Harap sediakan file gambar.",
        jpg_input_folder
    );
} else {
    process_folder(jpg_input_folder, jpg_output_folder).await?;
}

if !Path::new(png_input_folder).exists() {
    eprintln!(
        "Error: File input '{}' tidak ditemukan. Harap sediakan file gambar.",
        png_input_folder
    );
} else {
    process_folder(png_input_folder, png_output_folder).await?;
}

println!("Semua proses konversi selesai!");
Ok(())*/

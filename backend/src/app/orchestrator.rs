use crate::app::coordinator;
use crate::common::utils::random_sleep_time;
use crate::db::db::{DatabaseService, MangaSeries};
use crate::db::storage::StorageClient;
use crate::scraping::model::AppConfig;
use crate::scraping::{fetcher, parser};
use anyhow::{Result, anyhow};
use reqwest::Client;
use std::sync::Arc;

/// The main "engine" for a bulk scraping task.
/// This function can be called from anywhere, including a background task.
pub async fn run_bulk_series_scraping(
    series: MangaSeries,
    http_client: Client,
    db_service: &DatabaseService,
    app_config: Arc<AppConfig>,
    storage_client: Arc<StorageClient>,
) -> Result<()> {
    println!("[BULK SCRAPE] Starting for series: '{}'", series.title);

    // 1. Get the series main page URL from the provided struct
    let series_main_page_url = series
        .current_source_url
        .as_ref()
        .ok_or_else(|| anyhow!("Series source URL does not exist for '{}'", series.title))?;

    // 2. Get the correct site config
    let host = series
        .source_website_host
        .as_ref()
        .ok_or_else(|| anyhow!("Host not found for series '{}'", series.title))?;

    let site_config = app_config
        .get_site_config(host)
        .ok_or_else(|| anyhow!("No scraping config for host: {}", host))?;

    // 3. Fetch and Parse Series Main Page to Get Chapter List
    println!(
        "[BULK SCRAPE] Fetching series main page HTML from: {}",
        series_main_page_url
    );
    let series_page_html = fetcher::fetch_html(&http_client, series_main_page_url).await?;
    // Add a delay after fetching the main series page to avoid rapid requests
    random_sleep_time(3, 7).await;

    // 4. Extract all chapter links from the series main page
    let all_available_chapters = parser::extract_chapter_links(
        &series_page_html,
        series_main_page_url, // Series page URL, to create absolute URLs if needed
        site_config,          // Pass reference to the relevant site configuration
    )
    .await?;

    if all_available_chapters.is_empty() {
        println!(
            "[BULK SCRAPE] No chapters found on the series page for '{}'.",
            series.title
        );
        return Ok(()); // Not an error, just nothing to do.
    }
    println!(
        "[BULK SCRAPE] Found {} total chapters for '{}'.",
        all_available_chapters.len(),
        series.title
    );

    // 5. Determine which chapters to scrape. For bulk scrape, we take all of them
    // that are newer than what we have.
    let last_chapter_num = series.last_chapter_found_in_storage.unwrap_or(0.0);
    let chapters_to_scrape: Vec<parser::ChapterInfo> = all_available_chapters
        .into_iter()
        .filter(|ch_info| ch_info.number > last_chapter_num)
        .collect();

    if chapters_to_scrape.is_empty() {
        println!(
            "[BULK SCRAPE] No new chapters to scrape for '{}'. All are up-to-date.",
            series.title
        );
        return Ok(());
    }
    println!(
        "[BULK SCRAPE] Will scrape {} new chapters.",
        chapters_to_scrape.len()
    );

    // 6. Start Scraping Process for Selected Chapters
    // Note: We are no longer saving to a local path, the coordinator will handle R2/CDN urls.
    // The coordinator now needs the database connection to save chapter info.
    let last_downloaded_chapter = coordinator::process_series_chapters_from_list(
        &series,
        &chapters_to_scrape,
        &http_client,
        &storage_client,
        site_config,
        db_service,
    )
    .await?;

    // 7. Update series data in the database after scraping is complete
    if let Some(last_chapter_num) = last_downloaded_chapter {
        db_service
            .update_series_last_chapter_found_in_storage(series.id, Some(last_chapter_num))
            .await?;
        println!(
            "[BULK SCRAPE] Updated last local chapter for '{}' to {}.",
            series.title, last_chapter_num
        );
    }

    Ok(())
}

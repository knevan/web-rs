use crate::common::utils::random_sleep_time;
use crate::db::db::ManhwaSeries;
use crate::scraping::model::SiteScrapingConfig;
use crate::scraping::{coordinator, fetcher, parser};
use anyhow::{Context, Result};
use reqwest::Client;
use std::path::Path;

pub async fn scrape_series(
    series: &ManhwaSeries,
    series_path: &Path,
    http_client: &Client,
    site_config: &SiteScrapingConfig,
    max_chapters_to_scrape: usize,
) -> Result<Option<f32>> {
    // Get the series main page URL
    let series_main_page_url = series
        .current_source_url
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Series source URL does not exist"))?;

    // Fetch and Parse Series Main Page to Get Chapter List
    println!(
        "[MAIN] Fetching series main page HTML from: {}",
        series_main_page_url
    );
    // Pass the shared http_client to fetch_html
    let series_page_html = fetcher::fetch_html(http_client, series_main_page_url)
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
            series.title
        );

        return Ok(None);
    }

    println!(
        "[SCRAPE] Found {} chapters for series '{}'.",
        all_available_chapters.len(),
        series.title
    );

    // Logic to Determine Which Chapters to Scrape
    // Example: scrape the first 2 chapters newer than what's already present, or the first 2 if none exist.
    // TODO)): Make the number of chapters to scrape configurable.
    let last_local_chapter_num = series.last_chapter_found_locally.unwrap_or(0.0);
    let chapters_to_scrape: Vec<parser::ChapterInfo> = all_available_chapters
        .into_iter()
        // Get newer chapters
        // .rev() // If you want from newest to oldest (might need to sort by number desc in parser first)
        .filter(|ch_info| ch_info.number > last_local_chapter_num)
        .take(max_chapters_to_scrape) // Take N chapters for testing (e.g., 2)
        .collect();

    if chapters_to_scrape.is_empty() {
        println!(
            "[SCRAPE] No new chapters to scrape for series '{}' (last local: {}).",
            series.title, last_local_chapter_num
        );
        return Ok(None);
    }

    println!(
        "[MAIN] Will scrape {} chapters for series '{}': {:?}",
        chapters_to_scrape.len(),
        series.title,
        chapters_to_scrape
            .iter()
            .map(|c| c.number)
            .collect::<Vec<f32>>()
    );

    // Start Scraping Process for Selected Chapters
    match coordinator::process_series_chapters_from_list(
        series,
        series_path,
        &chapters_to_scrape, // List of filtered chapters
        http_client,         // Pass the shared client
        site_config,         // Pass site configuration to coordinator
    )
    .await
    {
        Ok(last_downloaded_chapter) => {
            println!(
                "[SCRAPE] Batch scraping finished. Last chapter downloaded: {:?}",
                last_downloaded_chapter
            );
            Ok(last_downloaded_chapter)
        }
        Err(e) => {
            eprintln!(
                "[SCRAPE] An error occurred during the scraping process for series '{}': {}",
                series.title, e
            );
            Err(e)
        }
    }
}

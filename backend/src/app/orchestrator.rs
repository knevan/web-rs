use crate::app::coordinator;
use crate::common::utils::random_sleep_time;
use crate::database::storage::StorageClient;
use crate::database::{DatabaseService, Series};
use crate::scraping::model::SitesConfig;
use crate::scraping::parser::ChapterInfo;
use crate::scraping::{fetcher, parser};
use crate::task_workers::repair_chapter_worker::RepairChapterMsg;
use anyhow::{Result, anyhow};
use reqwest::Client;
use std::env;
use std::sync::Arc;
use url::Url;

/// The main "engine" for a bulk scraping task.
/// This function can be called from anywhere, including a background task.
pub async fn run_bulk_series_scraping(
    series: Series,
    http_client: Client,
    db_service: &DatabaseService,
    sites_config: Arc<SitesConfig>,
    storage_client: Arc<StorageClient>,
) -> Result<()> {
    println!("[BULK SCRAPE] Starting for series: '{}'", series.title);

    let series_main_page_url = &series.current_source_url;

    let host = &series.source_website_host;

    let site_config = sites_config
        .get_site_config(host)
        .ok_or_else(|| anyhow!("No scraping config for host: {}", host))?;

    println!(
        "[BULK SCRAPE] Fetching series main page HTML from: {series_main_page_url}"
    );

    let series_page_html =
        fetcher::fetch_html(&http_client, series_main_page_url).await?;

    random_sleep_time(3, 7).await;

    let all_available_chapter_links = parser::extract_chapter_links(
        &series_page_html,
        series_main_page_url,
        site_config,
    )
    .await?;

    if all_available_chapter_links.is_empty() {
        println!(
            "[BULK SCRAPE] No chapters found on the series page for '{}'.",
            series.title
        );
        return Ok(());
    }
    println!(
        "[BULK SCRAPE] Found {} total chapters for '{}'.",
        all_available_chapter_links.len(),
        series.title
    );

    // Determine which chapters to scrape.
    // For bulk scrape, we take all of them that are newer than what we have.
    let last_chapter_number =
        series.last_chapter_found_in_storage.unwrap_or(0.0);

    let chapters_to_scrape: Vec<ChapterInfo> = all_available_chapter_links
        .into_iter()
        .filter(|ch_info| ch_info.number > last_chapter_number)
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

    // Start Scraping Process for Selected Chapters
    let last_info_downloaded_chapter =
        coordinator::process_series_chapters_from_list(
            &series,
            &chapters_to_scrape,
            &http_client,
            storage_client,
            site_config,
            db_service,
        )
        .await?;

    // Update series data in the database
    if let Some(last_chapter_num) = last_info_downloaded_chapter {
        db_service
            .update_series_last_chapter_found_in_storage(
                series.id,
                Some(last_chapter_num),
            )
            .await?;
        println!(
            "[BULK SCRAPE] Updated last local chapter for '{}' to {}.",
            series.title, last_chapter_num
        );
    }

    Ok(())
}

pub async fn repair_specific_chapter_series(
    msg: RepairChapterMsg,
    db_service: &DatabaseService,
    storage_client: Arc<StorageClient>,
    http_client: Client,
    sites_config: Arc<SitesConfig>,
) -> Result<()> {
    println!(
        "[REPAIR] Repairing chapter {} in series {}.",
        msg.chapter_number, msg.series_id
    );

    let series = db_service
        .get_manga_series_by_id(msg.series_id)
        .await?
        .ok_or_else(|| {
            anyhow!("Series with ID {} not found.", msg.series_id)
        })?;

    let image_urls_to_delete = db_service
        .get_images_urls_for_chapter_series(msg.series_id, msg.chapter_number)
        .await?;

    if !image_urls_to_delete.is_empty() {
        println!(
            "[REPAIR] Found {} images to delete from storage",
            image_urls_to_delete.len()
        );

        let public_cdn_url = env::var("PUBLIC_CDN_URL")?;
        let key_to_delete: Vec<String> = image_urls_to_delete
            .into_iter()
            .filter_map(|url| {
                url.strip_prefix(&format!("{}/", public_cdn_url))
                    .map(String::from)
            })
            .collect();

        storage_client.delete_image_objects(&key_to_delete).await?;
        println!("[REPAIR] Deleted images from storage");
    }

    db_service
        .delete_chapter_and_images_for_chapter(series.id, msg.chapter_number)
        .await?;
    println!("[REPAIR] Successfully delete old data from database");

    let new_host = Url::parse(&msg.new_chapter_url)?
        .host_str()
        .ok_or_else(|| {
            anyhow!("Invalid new chapter URL: {}", &msg.new_chapter_url)
        })?
        .to_string();

    let site_config = sites_config
        .get_site_config(&new_host)
        .ok_or_else(|| anyhow!("No scraping config for host: {}", new_host))?;

    let chapter_info_to_scrape = ChapterInfo {
        title: msg
            .new_chapter_title
            .map(|s| s.to_string())
            .unwrap_or_default(),
        url: msg.new_chapter_url.to_string(),
        number: msg.chapter_number,
    };

    coordinator::process_single_chapter(
        &series,
        &chapter_info_to_scrape,
        &http_client,
        storage_client,
        site_config,
        db_service,
    )
    .await?;

    println!(
        "[REPAIR] Repaired chapter {} in series {}.",
        msg.chapter_number, msg.series_id
    );

    Ok(())
}

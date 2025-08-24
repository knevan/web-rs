use crate::app::coordinator;
use crate::common::utils::random_sleep_time;
use crate::database::storage::StorageClient;
use crate::database::{DatabaseService, Series};
use crate::scraping::fetcher;
use crate::scraping::model::SitesConfig;
use crate::scraping::parser::{ChapterInfo, ChapterParser};
use crate::task_workers::repair_chapter_worker::RepairChapterMsg;
use anyhow::{Result, anyhow};
use arc_swap::ArcSwap;
use reqwest::Client;
use std::env;
use std::sync::Arc;
use url::Url;

// The main "engine" for checking series and scraping task.
// This function can be called from anywhere, including a background task.
pub async fn run_series_check(
    series: Series,
    http_client: Client,
    db_service: &DatabaseService,
    sites_config: Arc<SitesConfig>,
    storage_client: Arc<StorageClient>,
) -> Result<()> {
    println!("[SERIES CHECK] Starting for series: '{}'", series.title);

    let host = &series.source_website_host;
    let site_config = sites_config
        .get_site_config(host)
        .ok_or_else(|| anyhow!("No scraping config for host: {}", host))?;

    // Initialize parser once. It holds compiled selectors and regexes.
    let chapter_parser = ChapterParser::new(site_config.clone())?;

    println!(
        "[SERIES CHECK] Fetching series main page HTML from: {}",
        series.current_source_url
    );

    let series_page_html =
        fetcher::fetch_html(&http_client, &series.current_source_url).await?;

    random_sleep_time(2, 5).await;

    // [Quick Check] Get latest chapter
    println!("[SERIES CHECK] Performing quick check, get latest chapter.");
    let latest_site_chapter = chapter_parser
        .quick_check_extract_latest_chapter_info(
            &series_page_html,
            &series.current_source_url,
        )?;

    let last_db_chapter_number =
        series.last_chapter_found_in_storage.unwrap_or(0.0);
    let mut chapters_to_scrape: Vec<ChapterInfo> = Vec::new();
    let mut needs_full_scan = false;

    if let Some(latest_chapter) = latest_site_chapter {
        println!(
            "[SERIES CHECK] Latest on site: {:.2}, latest in DB: {:.2}",
            latest_chapter.number, last_db_chapter_number
        );

        // If latest chapter on site > latest in DB, we need a full scan.
        if latest_chapter.number > last_db_chapter_number {
            println!(
                "[SERIES CHECK] New chapter detected by Quick Check. Triggering full scan."
            );
            needs_full_scan = true;
        } else {
            // [Count Check] If no new chapter, check for backfills or deletions
            println!(
                "[SERIES CHECK] Quick Check passed. Performing Count Check"
            );
            let site_chapter_count =
                chapter_parser.count_chapter_links(&series_page_html)?;
            let db_chapter_count =
                db_service.get_series_chapters_count(series.id).await?;

            println!(
                "[SERIES CHECK] Chapter on site: {}, chapters in DB: {}",
                site_chapter_count, db_chapter_count
            );

            if site_chapter_count != db_chapter_count as usize {
                println!(
                    "[SERIES CHECK] Count missmatch. Trigger full scan for synchronization."
                );
                needs_full_scan = true;
            }
        }
    } else {
        println!("[SERIES CHECK] Couldnt found any chapter on the site.");
        return Ok(());
    }

    // [Full Scan] Only run if triggered by one of the checks above.
    if needs_full_scan {
        println!("[SERIES CHECK] Run full scan");
        let all_available_chapters = chapter_parser
            .full_scan_extract_all_chapter_info(
                &series_page_html,
                &series.current_source_url,
            )?;

        if all_available_chapters.is_empty() {
            println!(
                "[SERIES CHECK] Full scan found no chapters for '{}'.",
                series.title
            );
            return Ok(());
        }

        println!(
            "[SERIES CHECK] Full scan found {} unique chapters.",
            all_available_chapters.len()
        );

        // Filter chapters that are actually new to avoid re-scraping on a syncronization.
        chapters_to_scrape = all_available_chapters
            .into_iter()
            .filter(|ch_info| ch_info.number > last_db_chapter_number)
            .collect();
    }

    if chapters_to_scrape.is_empty() {
        println!(
            "[SERIES CHECK] No new chapters to scrape for '{}'. All are up-to-date.",
            series.title
        );
        return Ok(());
    }

    println!(
        "[SERIES CHECK] Found {} new chapters to scrape.",
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

    // Update series metadata in the database
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

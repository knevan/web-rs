use crate::common::utils::sanitize_series_title;
use crate::db::db::{
    ManhwaSeries, add_manhwa_series, current_timestamp, get_manhwa_series_by_id,
    get_manhwa_series_by_title, update_series_check_schedule, update_series_last_local_chapter,
    update_series_source_urls,
};
use anyhow::{Context, Result, anyhow};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

pub fn get_or_create_series(
    conn: &Connection,
    title: &str,
    default_url: &str,
    check_interval: i32,
) -> Result<ManhwaSeries> {
    match get_manhwa_series_by_title(conn, title)? {
        Some(mut series) => {
            println!(
                "[MAIN] Series '{}' found in database (ID: {}).",
                series.title, series.id
            );
            // Update source URL if different or empty in DB
            if series
                .current_source_url
                .clone()
                .is_none_or(|s| s.trim().is_empty() || s != default_url)
            {
                println!(
                    "[MAIN] Source URL for '{}' is different or empty in DB, updating with default URL: {}",
                    series.title, default_url
                );
                update_series_source_urls(&conn, series.id, default_url)?;
                // Update local series data after DB change
                series.current_source_url = Some(default_url.to_string());
                series.source_website_host = Url::parse(default_url)
                    .ok()
                    .and_then(|u| u.host_str().map(String::from));
            }
            Ok(series)
        }
        None => {
            println!("[MAIN] Series '{}' not found in database. Adding...", title);
            let new_series_id = add_manhwa_series(&conn, title, Some(default_url), check_interval)?;
            println!(
                "[MAIN] Series '{}' added successfully with ID: {}.",
                title, new_series_id
            );
            // Fetch the newly added series data for consistency
            get_manhwa_series_by_id(conn, new_series_id as i32)?
                .ok_or_else(|| anyhow!("Newly added series should exist."))
        }
    }
}

pub fn prepare_series_directory(root_dir: &Path, series: &ManhwaSeries) -> Result<PathBuf> {
    // Prepare Local Folder Path for This Series (LOCAL TEST ONLY)
    let series_folder_name = sanitize_series_title(&series.title);
    let series_base_path = root_dir.join(&series_folder_name);

    if !series_base_path.exists() {
        fs::create_dir_all(&series_base_path)
            .with_context(|| format!("Failed to create series folder: {:?}", series_base_path))?;
        println!("[MAIN] Series folder created: {:?}", series_base_path);
    }

    Ok(series_base_path)
}

pub fn get_series_host(series: &ManhwaSeries) -> Result<String> {
    // Get Specific Scraping Configuration for the Site from the Series URL
    let series_main_page_url = series.current_source_url.as_ref().ok_or_else(|| {
        anyhow::anyhow!("Series source URL does not exist for '{}'", series.title)
    })?;

    let series_host = Url::parse(series_main_page_url)?
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("Could not get host from URL: {}", series_main_page_url))?
        .to_string();

    Ok(series_host)
}

pub fn update_series_after_scraping(
    conn: &Connection,
    series: &mut ManhwaSeries,
    last_downloaded_chapter: Option<f32>,
) -> Result<()> {
    // Update the last locally found chapter in DB if a new one was downloaded
    if let Some(last_chapter_num) = last_downloaded_chapter {
        let last_storage_chapter_num = series.last_chapter_found_locally.unwrap_or(0.0);
        if last_chapter_num > last_storage_chapter_num {
            println!(
                "[MAIN] Updating last_chapter_found_locally in DB for series '{}' to {}.",
                series.title, last_chapter_num
            );
            update_series_last_local_chapter(&conn, series.id, Some(last_chapter_num))?;
            // Update series_data in memory as well
            series.last_chapter_found_locally = Some(last_chapter_num);
        }
    }

    // Update check schedule in database
    // Mark that this series has been processed and when to check it again.
    let current_ts = current_timestamp();
    // TODO)): The next check time should be calculated based on series_data.check_interval_minutes
    // For now, just updating last_processed_timestamp and status.
    // The `next_check_timestamp` should ideally be `current_ts + interval`
    update_series_check_schedule(&conn, series.id, Some("monitoring"), Some(current_ts), None)?;
    println!(
        "[MAIN] Check schedule for series '{}' has been updated.",
        series.title
    );

    println!("\n[MAIN] Scraper application finished.");
    Ok(())
}

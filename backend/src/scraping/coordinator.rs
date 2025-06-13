use crate::core::utils::{download_and_encode_image, random_sleep_time};
use crate::db::db::ManhwaSeries;
use crate::scraping::model::SiteScrapingConfig;
use crate::scraping::{fetcher, parser};
use anyhow::{Context, Result};
use reqwest::Client;
use std::fs;
use std::path::Path;

/// Process scraping and downloading single chapters
pub async fn process_single_chapter(
    series_title: &str,          // Series title (for logging and folder name)
    chapter_number_float: f32,   // Chapter number (for folder name and DB update)
    chapter_url: &str,           // Valid URL to the chapter page
    series_base_path: &Path,     // Base path of the series folder where chapters will be saved
    http_client: &Client,        // HTTP client to be used
    config: &SiteScrapingConfig, // Site-specific scraping configuration
) -> Result<Option<f32>> {
    // Returns Some(chapter_number) if successful, None otherwise
    let chapter_number_for_log = format!("{:.1}", chapter_number_float); // Format chapter number for logging
    println!(
        "\n[COORDINATOR] Processing Chapter {} (URL: {}) for series '{}'...",
        chapter_number_for_log, chapter_url, series_title
    );

    // Fetch HTML from the chapter page
    println!(
        "[COORDINATOR] Fetching HTML for Chapter {} from {}.",
        chapter_number_for_log, chapter_url
    );
    let html_content = match fetcher::fetch_html(http_client, chapter_url).await {
        // Pass the Client
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "[COORDINATOR] Failed to fetch HTML for Chapter {}: {}. Skipping this chapter.",
                chapter_number_for_log, e
            );
            return Ok(None); // Fetch failed, no images downloaded, but not a fatal error for the whole process
        }
    };
    // Short pause after fetching chapter page HTML
    //tokio::time::sleep(Duration::from_secs(config.host_name.contains("mgeko") as u64 * 2)).await;
    let base_delay_seconds = if config.host_name.contains("mgeko") {
        3
    } else {
        2
    };
    random_sleep_time(base_delay_seconds, base_delay_seconds + 3).await;

    // Parse HTML to get image URLs using site configuration
    let image_urls = match parser::extract_image_urls(
        &html_content,
        chapter_url, // Chapter page URL as base for relative image URLs
        config,      // Pass site configuration
    ) {
        Ok(urls) if urls.is_empty() => {
            println!(
                "[COORDINATOR] No image URLs found for Chapter {}.",
                chapter_number_for_log
            );
            return Ok(None); // No img URLs
        }
        Ok(urls) => urls,
        Err(e) => {
            eprintln!(
                "[COORDINATOR] Failed to parse images for Chapter {}: {}. Skipping this chapter.",
                chapter_number_for_log, e
            );
            return Ok(None); // Parse failed
        }
    };

    // 3. Create folder for this chapter if it doesn't exist
    // Using format Chapter_X or Chapter_X_Y for chapters with decimals
    let chapter_folder_name = format!("Chapter_{}", chapter_number_for_log.replace('.', "_"));
    let chapter_path = series_base_path.join(chapter_folder_name);
    if !chapter_path.exists() {
        fs::create_dir_all(&chapter_path).with_context(|| {
            format!(
                "Failed to create folder for Chapter {}: {:?}",
                chapter_number_for_log, chapter_path
            )
        })?;
        println!(
            "[COORDINATOR] Chapter {} folder created at: {:?}",
            chapter_number_for_log, chapter_path
        );
    }

    // Download and save each image
    // TODO)): Consider parallelizing image downloads for a single chapter (e.g., using tokio::spawn with a semaphore)
    println!(
        "[COORDINATOR] Downloading {} images for Chapter {}...",
        image_urls.len(),
        chapter_number_for_log
    );

    // Short pause before starting to download images
    random_sleep_time(2, 4).await;

    let mut images_downloaded_in_chapter_count = 0;
    for (index, img_url) in image_urls.iter().enumerate() {
        let image_filename = format!("{:03}.avif", index + 1); // Image filename: 001.jpg, 002.png, etc.
        let image_save_path = chapter_path.join(&image_filename);

        // Try to download and save the image
        if let Err(e) = download_and_encode_image(http_client, img_url, &image_save_path).await {
            eprintln!(
                "[COORDINATOR] Failed to download/save image {} (URL: {}) for Chapter {}: {}",
                image_filename, img_url, chapter_number_for_log, e
            );
            // Consider whether to stop this chapter if one image fails, or retry. For now, we continue.
        } else if image_save_path.exists() {
            images_downloaded_in_chapter_count += 1;
        }
    }

    println!(
        "[COORDINATOR] Finished processing Chapter {}. {} of {} images successfully downloaded.",
        chapter_number_for_log,
        images_downloaded_in_chapter_count,
        image_urls.len()
    );
    if images_downloaded_in_chapter_count > 0 {
        Ok(Some(chapter_number_float))
    } else {
        Ok(None)
    }
}

pub async fn process_series_chapters_from_list(
    series_data: &ManhwaSeries, // Manhwa series data from the database
    series_base_path: &Path,    // Base path of the series folder
    chapters_to_process: &[parser::ChapterInfo], // List of chapters to process (from parser::extract_chapter_links)
    http_client: &Client,                        // HTTP client
    config: &SiteScrapingConfig,                 // Scraping configuration for this site
) -> Result<Option<f32>> {
    // Returns the number of the last successfully downloaded chapter in this run
    println!(
        "[COORDINATOR] Starting processing for series '{}' with {} chapters to process..",
        series_data.title,
        chapters_to_process.len()
    );

    let mut last_successfully_downloaded_chapter_this_run: Option<f32> = None;

    if chapters_to_process.is_empty() {
        println!(
            "[COORDINATOR] No chapters to process for series '{}'.",
            series_data.title
        );
        return Ok(None);
    }

    // Iterate through each chapter that needs processing
    for chapter_info in chapters_to_process {
        // Call the function that processes a single chapter
        match process_single_chapter(
            &series_data.title,
            chapter_info.number, // Use chapter number from ChapterInfo
            &chapter_info.url,   // Use chapter URL from ChapterInfo
            series_base_path,
            http_client,
            config, // Pass site configuration
        )
        .await
        {
            Ok(Some(_)) => {
                // Some(_) means at least 1 image was downloaded for this chapter
                last_successfully_downloaded_chapter_this_run = Some(chapter_info.number);
                println!(
                    "[COORDINATOR] Chapter {:.1} ('{}') processed successfully.",
                    chapter_info.number, chapter_info.title
                );
            }
            Ok(None) => {
                println!(
                    "[COORDINATOR]  No images were downloaded for chapter {:.1} ('{}'). Chapter might be empty or all image failed.",
                    chapter_info.number, chapter_info.title
                );
            }
            Err(e) => {
                // Errors from process_single_chapter are usually logged there.
                // We can log again here or decide to stop the entire series processing.
                // For now, we continue to the next chapter.
                eprintln!(
                    "[COORDINATOR] Significant error while processing chapter {:.1} ('{}'): {}. Continuing to the next chapter if any.",
                    chapter_info.number, chapter_info.title, e
                );
            }
        }

        // Short pause (sleep) before processing next chapter
        println!("[COORDINATOR] Pausing before the next chapter (if any)...");
        //tokio::time::sleep(Duration::from_secs(config.host_name.contains("mgeko") as u64 * 8)).await;
        let base_chapter_delay_seconds = if config.host_name.contains("mgeko") {
            6
        } else {
            4
        };
        random_sleep_time(base_chapter_delay_seconds, base_chapter_delay_seconds + 6).await;
    }

    if last_successfully_downloaded_chapter_this_run.is_some() {
        println!(
            "[COORDINATOR] Series processing for '{}' finished. Last chapter successfully downloaded in this session: {:.1}",
            series_data.title,
            last_successfully_downloaded_chapter_this_run.unwrap()
        );
    } else {
        println!(
            "[COORDINATOR] Series processing for '{}' finished. No new chapters were successfully downloaded in this session.",
            series_data.title
        );
    }

    Ok(last_successfully_downloaded_chapter_this_run)
}

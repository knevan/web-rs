use anyhow::Result;
use reqwest::Client;
use slug::slugify;
use tokio::task;

use crate::common::utils::random_sleep_time;
use crate::db::db::{DatabaseService, MangaSeries};
use crate::db::storage::StorageClient;
use crate::encoding::image_encoding;
use crate::scraping::model::SiteScrapingConfig;
use crate::scraping::{fetcher, parser};

// Loops through a list of chapters and processes them one by one.
pub async fn process_series_chapters_from_list(
    series_data: &MangaSeries,
    chapters_to_process: &[parser::ChapterInfo],
    http_client: &Client,
    storage_client: &StorageClient,
    config: &SiteScrapingConfig,
    db_service: &DatabaseService,
) -> Result<Option<f32>> {
    println!(
        "[COORDINATOR] Starting batch processing for '{}'.",
        series_data.title
    );

    let mut last_successfully_downloaded_chapter: Option<f32> = None;

    for chapter_info in chapters_to_process {
        match process_single_chapter(
            series_data,
            chapter_info,
            http_client,
            storage_client,
            config,
            db_service,
        )
        .await
        {
            Ok(Some(chapter_num)) => {
                last_successfully_downloaded_chapter = Some(chapter_num);
            }
            Ok(None) => { /* Chapter was processed but had no images, which is fine */
            }
            Err(e) => {
                eprintln!(
                    "[COORDINATOR] Error processing chapter {}, stopping series: {}",
                    chapter_info.number, e
                );
                // Decide if you want to stop the whole process on a single chapter failure
                // break;
            }
        }
        random_sleep_time(6, 12).await; // Pause between chapters
    }
    Ok(last_successfully_downloaded_chapter)
}

/// Process scraping and downloading single chapters
pub async fn process_single_chapter(
    series: &MangaSeries,
    chapter_info: &parser::ChapterInfo,
    http_client: &Client,
    storage_client: &StorageClient,
    config: &SiteScrapingConfig,
    db_service: &DatabaseService,
) -> Result<Option<f32>> {
    println!(
        "[COORDINATOR] Processing Chapter {} for '{}'...",
        chapter_info.number, series.title
    );

    let chapter_id = db_service
        .add_new_chapter(
            series.id,
            chapter_info.number,
            Some(&chapter_info.title),
            &chapter_info.url,
        )
        .await?;
    println!(
        "[COORDINATOR] Chapter {} saved to DB with ID: {}",
        chapter_info.number, chapter_id
    );

    let html_content =
        fetcher::fetch_html(http_client, &chapter_info.url).await?;

    random_sleep_time(3, 5).await;

    let image_urls = parser::extract_image_urls_from_html_content(
        &html_content,
        &chapter_info.url,
        config,
    )?;
    if image_urls.is_empty() {
        println!(
            "[COORDINATOR] No image URLs found for Chapter {}.",
            chapter_info.number
        );
        return Ok(None);
    }

    // Process each image
    let mut image_saved_count = 0;
    let series_slug = slugify(&series.title);

    for (index, img_url) in image_urls.iter().enumerate() {
        random_sleep_time(2, 4).await;

        // store image to R2 object storage
        let final_cdn_url = match fetcher::fetch_image_bytes(
            http_client,
            img_url,
        )
        .await
        {
            Ok(image_bytes) => {
                let avif_bytes_result = task::spawn_blocking(move || {
                    image_encoding::covert_image_bytes_to_avif(&image_bytes)
                })
                .await?;

                match avif_bytes_result {
                    Ok(avif_bytes) => {
                        // Define the key for the object in R2
                        let object_key = format!(
                            "{}/{}/{:03}.avif",
                            series_slug,
                            chapter_info.number,
                            index + 1
                        );

                        // Upload to R2
                        match storage_client
                            .upload_image_objects(
                                &object_key,
                                avif_bytes,
                                "image/avif",
                            )
                            .await
                        {
                            Ok(cdn_url) => Some(cdn_url),
                            Err(e) => {
                                eprintln!(
                                    "[COORDINATOR] Failed to upload to R2: {e}"
                                );
                                None
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "[COORDINATOR] Failed to encode image to AVIF: {e}"
                        );
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "[COORDINATOR] Failed to fetch image bytes from {img_url}: {e}",
                );
                None
            }
        };

        // Save CDN Url to the database if successful
        if let Some(cdn_url) = final_cdn_url {
            if db_service
                .add_chapter_images(chapter_id, (index + 1) as i32, &cdn_url)
                .await
                .is_ok()
            {
                image_saved_count += 1;
            }
        }
    }

    println!(
        "[COORDINATOR] Finished Chapter {}. {} of {} images saved to storage.",
        chapter_info.number,
        image_saved_count,
        image_urls.len()
    );

    if image_saved_count > 0 {
        Ok(Some(chapter_info.number))
    } else {
        Ok(None)
    }
}

use anyhow::Result;
use reqwest::Client;
use slug::slugify;
use std::sync::Arc;
use tokio::task;

use crate::common::utils::random_sleep_time;
use crate::database::storage::StorageClient;
use crate::database::{DatabaseService, Series};
use crate::encoding::image_encoding;
use crate::scraping::model::SiteScrapingConfig;
use crate::scraping::{fetcher, parser};

// Manage loop through a list of chapters and processes them one by one.
pub async fn process_series_chapters_from_list(
    series_data: &Series,
    chapters_to_process: &[parser::ChapterInfo],
    http_client: &Client,
    storage_client: Arc<StorageClient>,
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
            storage_client.clone(),
            config,
            db_service,
        )
        .await
        {
            Ok(Some(chapter_num)) => {
                last_successfully_downloaded_chapter = Some(chapter_num);
            }
            Ok(None) => {
                /* Chapter was processed but had no images, which is fine don't stop process */
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
        // Pause between scraping chapters
        random_sleep_time(4, 8).await;
    }
    Ok(last_successfully_downloaded_chapter)
}

// Process scraping and downloading single chapters
pub async fn process_single_chapter(
    series: &Series,
    chapter_info: &parser::ChapterInfo,
    http_client: &Client,
    storage_client: Arc<StorageClient>,
    config: &SiteScrapingConfig,
    db_service: &DatabaseService,
) -> Result<Option<f32>> {
    let convert_chapter_number =
        chapter_info.number.to_string().replace('.', "-");

    let consistent_title = format!("{}-eng", convert_chapter_number);

    println!(
        "[COORDINATOR] Processing Chapter {} for '{}'...",
        chapter_info.number, series.title
    );

    let chapter_id = db_service
        .add_new_chapter(
            series.id,
            chapter_info.number,
            Some(&consistent_title),
            &chapter_info.url,
        )
        .await?;
    println!(
        "[COORDINATOR] Chapter {} saved to DB with ID: {}",
        chapter_info.number, chapter_id
    );

    let html_content =
        fetcher::fetch_html(http_client, &chapter_info.url).await?;

    // Pause before start processing images
    random_sleep_time(2, 4).await;

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
        // Pause between image downloads
        random_sleep_time(2, 4).await;

        // store image to R2 object storage
        let image_store_result = match fetcher::fetch_image_bytes(
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
                        // domain/{series-name}/{chapter-number}/{image-number}.avif
                        let object_key = format!(
                            "series/{}/ch-{}/{:03}.avif",
                            series_slug,
                            chapter_info.number.to_string().replace('.', "-"),
                            index
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
                            Ok(_) => Some(object_key),
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

        // Save CDN object key to the database if successful
        if let Some(key_to_save) = image_store_result
            && db_service
                .add_chapter_images(
                    chapter_id,
                    (index + 1) as i32,
                    &key_to_save,
                )
                .await
                .is_ok()
        {
            image_saved_count += 1;
        }
    }

    println!(
        "[COORDINATOR] Finished Chapter {}. {} of {} images saved to storage.",
        chapter_info.number,
        image_saved_count,
        image_urls.len()
    );

    if image_saved_count > 0 {
        // If images were saved, it means new content was added.
        if let Err(e) = db_service
            .update_series_new_content_timestamp(series.id)
            .await
        {
            eprintln!(
                "Failed to update series content timestamp for series_id {}: {}",
                series.id, e
            );
        }
        Ok(Some(chapter_info.number))
    } else {
        Ok(None)
    }
}

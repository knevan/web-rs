use std::sync::Arc;

use anyhow::Result;
use reqwest::Client;
use slug::slugify;
use tokio::sync::Semaphore;
use tokio::task;

use crate::common::utils::random_sleep_time;
use crate::database::storage::StorageClient;
use crate::database::{ChapterStatus, DatabaseService, Series};
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
        random_sleep_time(3, 6).await;
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
    let convert_chapter_number = chapter_info.number.to_string().replace('.', "-");

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

    let html_content = fetcher::fetch_html(http_client, &chapter_info.url).await?;

    // Pause before start processing images
    random_sleep_time(1, 3).await;

    let image_urls = parser::extract_image_urls_from_html_content(
        &html_content,
        &chapter_info.url,
        config,
    )?;

    let total_image_found = image_urls.len();

    if image_urls.is_empty() {
        println!(
            "[COORDINATOR] No image URLs found for Chapter {}.",
            chapter_info.number
        );
        return Ok(None);
    }

    let semaphore = Arc::new(Semaphore::new(2));
    let series_slug = slugify(&series.title);
    let mut processing_tasks = Vec::new();

    // Process image
    for (index, img_url) in image_urls.into_iter().enumerate() {
        let http_client = http_client.clone();
        let storage_client = storage_client.clone();
        let series_slug = series_slug.clone();
        let chapter_number_str = convert_chapter_number.clone();
        let permit_semaphore = Arc::clone(&semaphore);

        let task = tokio::spawn(async move {
            // This will wait until a permit is available from the semaphore
            let _permit = permit_semaphore.acquire_owned().await.unwrap();

            // Pause random delay before task
            //random_sleep_time(1, 2).await;

            // The processing pipeline: fetch -> encode -> upload
            let image_bytes =
                match fetcher::fetch_image_bytes(&http_client, &img_url).await {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        eprintln!(
                            "[COORDINATOR-TASK][Ch:{}/Img:{}] Failed to fetch {}: {}",
                            chapter_number_str, index, img_url, e
                        );
                        return (index, Err(anyhow::anyhow!("Fetch failed")));
                    }
                };

            let avif_bytes = match task::spawn_blocking(move || {
                image_encoding::covert_image_bytes_to_avif(&image_bytes)
            })
            .await
            {
                Ok(Ok(bytes)) => bytes,
                Ok(Err(e)) => {
                    return (index, Err(anyhow::anyhow!("Encoding failed: {}", e)));
                }
                Err(e) => {
                    return (
                        index,
                        Err(anyhow::anyhow!("Encoding task panicked: {}", e)),
                    );
                }
            };

            // Define the key for the object in R2
            // domain/{series-name}/{chapter-number}/{image-number}.avif
            let object_key = format!(
                "series/{}/ch-{}/{:03}.avif",
                series_slug, chapter_number_str, index
            );

            // Upload to R2
            if let Err(e) = storage_client
                .upload_image_series_objects(&object_key, avif_bytes, "image/avif")
                .await
            {
                eprintln!("[TASK] Failed to upload to R2: {}", e);
                return (index, Err(anyhow::anyhow!("Upload failed")));
            }

            (index, Ok(object_key))
        });

        processing_tasks.push(task);
    }

    let task_results = futures::future::join_all(processing_tasks).await;

    let mut successful_uploads: Vec<(usize, String)> = Vec::new();
    for result in task_results {
        match result {
            // Task complete
            Ok((index, Ok(object_key))) => {
                successful_uploads.push((index, object_key));
            }
            // Task run but return error
            Ok((_, Err(task_err))) => {
                eprintln!("[COORDINATOR] Error processing task: {}", task_err);
            }
            // Task panicked
            Err(join_err) => {
                eprintln!("[COORDINATOR] Processing task panicked: {}", join_err);
            }
        }
    }

    // Sort the successful results by their original index to ensure correct page order
    successful_uploads.sort_by_key(|(index, _)| *index);

    // Perform the database writes sequentially and in the correct order
    for (original_index, key_to_save) in &successful_uploads {
        // Save CDN object key to the database if successful
        if db_service
            .add_chapter_images(chapter_id, (*original_index + 1) as i32, key_to_save)
            .await
            .is_err()
        {
            eprintln!(
                "[COORDINATOR] Failed to save image record to DB for key: {}",
                key_to_save
            );
            // This specific DB write failed, but we continue with the others.
        }
    }

    let image_saved_count = successful_uploads.len();

    println!(
        "[COORDINATOR] Finished Chapter {}. {} of {} images saved to storage.",
        chapter_info.number, image_saved_count, total_image_found
    );

    match (
        total_image_found > 0,
        image_saved_count == total_image_found,
    ) {
        (true, true) => {
            db_service
                .update_chapter_status(chapter_id, ChapterStatus::Available)
                .await?;

            if let Err(e) = db_service
                .update_series_new_content_timestamp(series.id)
                .await
            {
                eprintln!(
                    "Non-critical error: Failed to update series timestamp: {}",
                    e
                );
            }
            Ok(Some(chapter_info.number))
        }
        // Partial/Incomplete chapter images
        (true, false) => {
            db_service
                .update_chapter_status(chapter_id, ChapterStatus::Error)
                .await?;
            Ok(None)
        }
        // No images found
        (false, _) => {
            db_service
                .update_chapter_status(chapter_id, ChapterStatus::NoImagesFound)
                .await?;
            Ok(None)
        }
    }
}

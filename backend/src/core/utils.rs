use anyhow::{Context, Result};
use rand::Rng;
use reqwest::Client;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use tokio::{fs, task};
use url::Url;

use crate::encoding::image_encoding::covert_image_bytes_to_avif;

// Converts a relative URL string to an absolute URL string, given a base URL.
pub fn to_absolute_url(base_url_str: &str, relative_url_str: &str) -> Result<String> {
    let base_url = Url::parse(base_url_str)
        .with_context(|| format!("Base URL not valid: {}", base_url_str))?;

    let absolute_url = base_url.join(relative_url_str).with_context(|| {
        format!(
            "Failed to append URL: {} with {}",
            base_url_str, relative_url_str
        )
    })?;

    Ok(absolute_url.into()) // .into() is equivalent to .to_string() for Url
}

/// Pauses execution asynchronously for a random duration between `min_secs` and `max_secs`.
/// If `min_secs` is greater than or equal to `max_secs`, it sleeps for `min_secs`.
/// If the calculated sleep duration is 0, no sleep occurs.
pub async fn random_sleep_time(min_secs: u64, max_secs: u64) {
    let sleep_duration_seconds = if min_secs >= max_secs {
        min_secs
    } else {
        rand::rng().random_range(min_secs..=max_secs)
    };

    // If duration 0 don't sleep, otherwise sleep for a random duration
    if sleep_duration_seconds > 0 {
        sleep(Duration::from_secs(sleep_duration_seconds)).await;
    }
}

/// Downloads an image from a given URL using the provided `reqwest::Client` and saves it to `save_path`.
/// Skips download if the file at `save_path` already exists.
/// Creates parent directories for `save_path` if they don't exist.
/// Includes a fixed delay after successful download.
pub async fn download_and_convert_to_avif(
    client: &Client,
    url: &str,
    save_path: &Path,
) -> Result<()> {
    // Download image data into bytes
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to send request for image URL: {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Request for image {} failed with status: {}",
            url,
            response.status()
        ));
    }

    let image_bytes = response
        .bytes()
        .await
        .with_context(|| format!("Failed to read image bytes from {}", url))?
        .to_vec(); // Convert bytes to Vec

    println!(
        "[DOWNLOADER] Downloaded {} bytes from {}",
        image_bytes.len(),
        url
    );

    // Convert the downloaded bytes to AVIF bytes in a non-block
    // spawn_blocking is crucial here because image encoding is CPU-intensive and need time to complete
    let avif_bytes = task::spawn_blocking(move || covert_image_bytes_to_avif(&image_bytes))
        .await?
        .with_context(|| "The image conversion failed.")?;

    // Save the resulting AVIF bytes to the file system
    if let Some(parent_dir) = save_path.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)
                .await
                .with_context(|| format!("Failed to create parent directory: {:?}", parent_dir))?;
        }
    }

    fs::write(save_path, &avif_bytes)
        .await
        .with_context(|| format!("Failed to write AVIF data to: {:?}", save_path))?;

    println!(
        "[SAVER] Successfully saved converted AVIF image to: {:?}",
        save_path
    );

    // Consider making this delay configurable or part of random_sleep_time
    // Random delay after each image download.
    random_sleep_time(3, 6).await;
    Ok(())
}

/// Sanitizes a series title to be suitable for use as a folder name.
/// Replaces common problematic characters with hyphens or removes them.
pub fn sanitize_series_title(title: &str) -> String {
    title
        .replace([':', '/', '\\', '?', '*', '<', '>', '|'], "-") // Replace multiple characters with a single replacement
        .replace('\"', "'") // Replace double quotes with single quotes
        .trim() // Remove leading/trailing whitespace
        .to_string()
}

use crate::processing::image_processing;
use anyhow::{Context, Result};
use rand::Rng;
use reqwest::Client;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

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
pub async fn download_and_encode_image(
    client: &Client,
    image_url: &str,
    save_path: &Path,
) -> Result<()> {
    if save_path.exists() {
        println!(
            "[DOWNLOADER] Image already exists, skipping: {:?}",
            save_path
        );
        return Ok(());
    }
    println!("[DOWNLOADER] Downloading image from: {}", image_url);

    let random_request_timeout = Duration::from_secs(if 55 >= 70 {
        55
    } else {
        rand::rng().random_range(55..=75)
    });

    let response = client
        .get(image_url)
        .timeout(random_request_timeout) // Random timeout for the image download request
        .send()
        .await
        .with_context(|| format!("Failed to send GET request to image URL: {}", image_url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download image: '{}'. Server status: {}",
            image_url,
            response.status()
        ));
    }

    let image_bytes = response
        .bytes()
        .await
        .with_context(|| format!("Failed to read image bytes from {}", image_url))?
        .to_vec(); // Convert bytes to Vec

    println!("[ENCODER] Encoding image to AVIF for: {}", image_url);

    let avif_bytes = match image_processing::covert_to_avif_in_memory(image_bytes).await {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("[ENCODER] Failed to encode image from {}: {}", image_url, e);
            return Ok(());
        }
    };

    println!("[SAVER] Saving image AVIF to: {:?}", save_path);

    if let Some(parent_dir) = save_path.parent() {
        if !parent_dir.exists() {
            tokio::fs::create_dir_all(parent_dir)
                .await
                .with_context(|| format!("Failed to create parent directory: {:?}", parent_dir))?;
        }
    }

    tokio::fs::write(save_path, &avif_bytes)
        .await
        .with_context(|| format!("Failed to save image AVIF to: {:?}", save_path))?;

    println!("[SAVER] Successfully saved image AVIF to: {:?}", save_path);

    // Consider making this delay configurable or part of random_sleep_time
    // Random delay after each image download.
    random_sleep_time(3, 6).await;
    Ok(())
}

/// Sanitizes a series title to be suitable for use as a folder name.
/// Replaces common problematic characters with hyphens or removes them.
///
/// # Arguments
/// * `title`: The original series title string.
///
/// # Returns
/// `String`: A sanitized string suitable for a folder name.
pub fn sanitize_series_title(title: &str) -> String {
    title
        .replace([':', '/', '\\', '?', '*', '<', '>', '|'], "-") // Replace multiple characters with a single replacement
        .replace('\"', "'") // Replace double quotes with single quotes
        .trim() // Remove leading/trailing whitespace
        .to_string()
}

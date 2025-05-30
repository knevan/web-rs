use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::time::Duration;
use url::Url;

/// Convert a relative URL to an absolute URL
pub fn absolutify_url(base_url_str: &str, relative_url_str: &str) -> Result<String> {
    let base_url = Url::parse(base_url_str)
        .with_context(|| format!("Base URL not valid: {}", base_url_str))?;

    let absolute_url = base_url.join(relative_url_str).with_context(|| {
        format!(
            "Failed to append URL: {} with {}",
            base_url_str, relative_url_str
        )
    })?;

    Ok(absolute_url.to_string())
}

/// Download individual images from a given list of URLs and save them to a specified directory
pub async fn download_and_save_image(
    client: &reqwest::Client,
    image_url: &str,
    save_path: &Path,
) -> Result<()> {
    if save_path.exists() {
        println!("[DOWNLOADER] Image already exists: {:?}", save_path);
        return Ok(());
    }

    println!("[DOWNLOADER] Downloading image from: {}", image_url);
    let response = client
        .get(image_url)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .with_context(|| format!("Failed to send request to: {}", image_url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download image: {}, status: {}",
            image_url,
            response.status()
        ));
    }

    let image_bytes = response
        .bytes()
        .await
        .with_context(|| format!("Failed to read bytes from {}", image_url))?;

    if let Some(parent_dir) = save_path.parent() {
        fs::create_dir_all(parent_dir)
            .with_context(|| format!("Failed to create parent directory: {:?}", parent_dir))?;
    }

    fs::write(save_path, &image_bytes)
        .with_context(|| format!("Failed to save image to: {:?}", save_path))?;
    println!("[DOWNLOADER] Image saved to: {:?}", save_path);

    tokio::time::sleep(Duration::from_secs(3)).await;
    Ok(())
}

/// Sanitize series title into valid folder name
pub fn sanitize_series_title(title: &str) -> String {
    title
        .replace(":", "-")
        .replace("/", "-")
        .replace("\\", "-")
        .replace("?", "")
        .replace("*", "")
        .replace("\"", "'")
        .replace("<", "")
        .replace(">", "")
        .replace("|", "")
        .trim()
        .to_string()
}

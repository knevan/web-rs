use anyhow::Context;
use reqwest::Client;

// Fetches HTML content from a given URL using a provided HTTP client.
pub async fn fetch_html(client: &Client, url: &str) -> anyhow::Result<String> {
    println!("Fetching HTML from {}", url);
    // The client is now passed as an argument, so we don't build it here.
    // The user_agent and other client configurations should be set when the client is initially created (in main.rs or core::dynamic_proxy)

    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch HTML from {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch HTML from {}: {}",
            url,
            response.status()
        ));
    }

    let body = response
        .text()
        .await
        .with_context(|| format!("Failed to read response body from {}", url))?;

    println!("HTML fetched successfully");
    Ok(body)
}

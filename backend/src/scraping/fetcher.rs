//use std::fmt::format;
use anyhow::Context;

pub async fn fetch_html(url: &str) -> anyhow::Result<String> {
    println!("Fetching HTML from {}", url);
    let client = reqwest::Client::builder() 
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()?;
    
    let response = client.get(url).send().await 
        .with_context(|| format!("Failed to fetch HTML from {}", url))?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch HTML from {}: {}",
            url,
            response.status()
        ))
    }
    
    let body = response.text().await 
        .with_context(|| format!("Failed to read response body from {}", url))?;
    
    println!("HTML fetched successfully");
    Ok(body)
}
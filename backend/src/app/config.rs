use crate::scraping::model::{AppConfig, SiteScrapingConfig};
use anyhow::{Context, Result, anyhow};
use url::Url;

/// Loads the application configuration from the specified file
pub fn load_app_config(config_path: &str) -> Result<AppConfig> {
    println!("[CONFIG] Loading application configuration...");
    // Load Application Configuration from config.toml
    // This is the first crucial step to get scraping parameters.
    let app_config = AppConfig::load(config_path)
        .with_context(|| "Failed to load application configuration from config.toml. Ensure the file exists and its format is correct.")?;
    println!("[MAIN] Application configuration loaded successfully.");

    Ok(app_config)
}

/// Gets the site configuration for a specific URL
pub fn get_site_config_for_url<'a>(
    app_config: &'a AppConfig,
    url: &str,
) -> Result<&'a SiteScrapingConfig> {
    let parsed_url = Url::parse(url)?;
    let host = parsed_url
        .host_str()
        .ok_or_else(|| anyhow!("Could not get host from URL: {}", url))?
        .to_string();

    let site_config = app_config.get_site_config(&host)
                                .ok_or_else(|| anyhow::anyhow!("No scraping configuration found for host: {}. Ensure the host is listed in config.toml.", host))?;

    Ok(site_config)
}

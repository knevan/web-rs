use crate::scraping::model::{SiteScrapingConfig, SitesConfig};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use url::Url;

// Helper function to get the specific site config from the global AppConfig based on a URL.
pub fn get_site_config_for_url(
    sites_config: Arc<SitesConfig>,
    url: &str,
) -> Result<SiteScrapingConfig> {
    let parsed_url = Url::parse(url)?;
    let host = parsed_url
        .host_str()
        .ok_or_else(|| anyhow!("Could not get host from URL: {}", url))?
        .to_string();

    let site_config = sites_config.get_site_config(&host)
                                .ok_or_else(|| anyhow!("No scraping configuration found for host: {}. Ensure the host is listed in config_sites.toml.", host))?;

    Ok(site_config.clone())
}

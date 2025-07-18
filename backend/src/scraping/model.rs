use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Configuration for scraping a specific website.
#[derive(Deserialize, Clone, Debug)]
pub struct SiteScrapingConfig {
    pub host_name: String,
    pub chapter_link_selector: String, // CSS selector for chapter links on the series page
    pub chapter_number_from_url_regex: Option<String>, // Regex to extract chapter number from chapter URL
    pub chapter_number_from_text_regex: Option<String>, // Regex to extract chapter number from link text
    pub chapter_number_data_attribute_on_parent: Option<String>, // Data attribute on parent element for chapter number

    pub image_selector_on_chapter_page: String, // CSS selector for image elements on chapter page
    pub image_url_attribute: String, // Primary attribute to get image URL ("src", "data-src")
    pub image_url_fallback_attributes: Vec<String>, // Fallback attributes if primary fails

                                                    // Consider adding delay configurations here:
                                                    // pub delay_after_chapter_page_fetch_ms: Option<u64>,
                                                    // pub delay_after_image_download_ms: Option<u64>,
                                                    // pub delay_between_chapters_ms: Option<u64>,
}

/// Main application configuration, loaded from a TOML file.
#[derive(Deserialize)]
pub struct AppConfig {
    #[serde(rename = "sites")]
    pub site_configs: Vec<SiteScrapingConfig>,
}

impl AppConfig {
    /// Loads application configuration from the specified path.
    pub fn load(config_path_str: &str) -> Result<Self> {
        let config_path = Path::new(config_path_str);
        if !config_path.exists() {
            return Err(anyhow::anyhow!(
                "[CONFIG] File configuration not found: {}",
                config_path.display()
            ));
        }

        let config_content =
            fs::read_to_string(config_path).with_context(|| {
                format!(
                    "[CONFIG] Failed to read file: {}",
                    config_path.display()
                )
            })?;

        let app_config: AppConfig = toml::from_str(&config_content)
            .with_context(|| {
                format!(
                    "[CONFIG] Failed to parse TOML configuration: {}",
                    config_path.display()
                )
            })?;

        println!(
            "[CONFIG] Configuration loaded successfully {} site(s) from {}",
            app_config.site_configs.len(),
            config_path_str
        );
        Ok(app_config)
    }

    /// Retrieves site-specific scraping configuration based on hostname.
    pub fn get_site_config(
        &self,
        host_name: &str,
    ) -> Option<&SiteScrapingConfig> {
        self.site_configs
            .iter()
            .find(|sc| sc.host_name == host_name)
    }
}

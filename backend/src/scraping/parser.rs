use crate::core::utils;
use anyhow::Result;
use scraper::{Html, Selector};

pub fn extract_image_urls(
    html_content: &str,
    base_url_relative_path: &str,
    css_selector: &str,
) -> Result<Vec<String>> {
    println!("Parsing HTML content...");
    let document = Html::parse_document(html_content);
    let selector = Selector::parse(css_selector)
        .map_err(|e| anyhow::anyhow!("Selector CSS not valid: {}. Error: {:?}", css_selector, e))?;

    let mut image_urls = Vec::new();
    for element in document.select(&selector) {
        let mut image_source_found: Option<String> = None;

        if let Some(src) = element.value().attr("src") {
            let trimmed_src = src.trim();
            if !trimmed_src.is_empty() {
                match utils::absolutify_url(base_url_relative_path, trimmed_src) {
                    Ok(abs_url) => {
                        image_source_found = Some(abs_url);
                    }
                    Err(e) => {
                        eprintln!(
                            "[PARSER] Failed to create absolute URLs for src '{}': {}. Use valid original URLs",
                            trimmed_src, e
                        );
                        if url::Url::parse(trimmed_src).is_ok() {
                            image_source_found = Some(trimmed_src.to_string());
                        }
                    }
                }
            }
        }

        if image_source_found.is_none() {
            if let Some(data_src) = element.value().attr("data-src") {
                let trimmed_data_src = data_src.trim();
                if !trimmed_data_src.is_empty() {
                    match utils::absolutify_url(base_url_relative_path, trimmed_data_src) {
                        Ok(abs_url) => {
                            image_source_found = Some(abs_url);
                        }
                        Err(e) => {
                            eprintln!(
                                "[PARSER] Failed to create absolute URLs for data-src '{}': {}. Use valid data-src",
                                trimmed_data_src, e
                            );
                            if url::Url::parse(trimmed_data_src).is_ok() {
                                image_source_found = Some(trimmed_data_src.to_string());
                            }
                        }
                    }
                }
            }
        }

        if let Some(url_to_add) = image_source_found {
            if !url_to_add.is_empty() {
                image_urls.push(url_to_add)
            }
        }
    }
    println!("[PARSER] Found {} image URLs", image_urls.len());
    Ok(image_urls)
}

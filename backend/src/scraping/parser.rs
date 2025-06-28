use anyhow::Result;
use regex::Regex;
use scraper::{Element, Html, Selector};
use url::Url;

use crate::common::utils;
use crate::scraping::model::SiteScrapingConfig;

/// Holds information about a single chapter.
#[derive(Debug, Clone)]
pub struct ChapterInfo {
    pub title: String,
    pub url: String,
    pub number: f32,
}

/// Extracts image URLs from the HTML content of a chapter page.
pub fn extract_image_urls(
    html_content: &str,
    base_url_relative_path: &str, // Should be the chapter page URL
    config: &SiteScrapingConfig,
) -> Result<Vec<String>> {
    println!(
        "[PARSER] Parsing HTML for link chapter with selector: {}",
        config.image_selector_on_chapter_page
    );
    let document = Html::parse_document(html_content);

    let image_element_selector =
        Selector::parse(&config.image_selector_on_chapter_page).map_err(|e| {
            anyhow::anyhow!(
                "Invalid CSS selector for image: '{}'. Error: {:?}",
                config.image_selector_on_chapter_page,
                e
            )
        })?;

    let mut image_urls = Vec::new();

    for img_element in document.select(&image_element_selector) {
        let mut image_source_found: Option<String> = None;

        // Try the primary attribute specified in config
        if let Some(src_val) = img_element.value().attr(&config.image_url_attribute) {
            let trimmed_src = src_val.trim();
            if !trimmed_src.is_empty() {
                match utils::to_absolute_url(base_url_relative_path, trimmed_src) {
                    Ok(abs_url) => {
                        image_source_found = Some(abs_url);
                        println!(
                            "[PARSER] Found img url from attr {}: {}",
                            config.image_url_attribute, trimmed_src
                        )
                    }
                    Err(e) => {
                        eprintln!(
                            "[PARSER] Failed to absolute URL from primary attribute '{}' (value: '{}'): {}. Trying as is.",
                            config.image_url_attribute, trimmed_src, e
                        );
                        if Url::parse(trimmed_src).is_ok() {
                            image_source_found = Some(trimmed_src.to_string());
                        }
                    }
                }
            }
        }

        // If not found, try fallback attributes
        if image_source_found.is_none() {
            for fallback_attr in &config.image_url_fallback_attributes {
                if let Some(src_val) = img_element.value().attr(fallback_attr) {
                    let trimmed_src = src_val.trim();
                    if !trimmed_src.is_empty() {
                        match utils::to_absolute_url(base_url_relative_path, trimmed_src) {
                            Ok(abs_url) => {
                                image_source_found = Some(abs_url);
                                println!(
                                    "[PARSER] Found img URL from fallback attr '{}': {}",
                                    fallback_attr, trimmed_src
                                );
                                break; // Found one, no need to check other fallback for this element
                            }
                            Err(e) => {
                                eprintln!(
                                    "[PARSER] Failed to create absolute URLs from fallback attribute '{}' (value: '{}'): {}. Trying as is.",
                                    fallback_attr, trimmed_src, e
                                );
                                if Url::parse(trimmed_src).is_ok() {
                                    image_source_found = Some(trimmed_src.to_string());
                                    break; // Found one, no need to check other fallback for this element
                                }
                            }
                        }
                    }
                    if image_source_found.is_some() {
                        break;
                    }
                }
            }

            // If still not found, as a last resort, check common attributes like 'src' or 'data-src' if not already primary/fallback
            // This part is a bit redundant if config is comprehensive, but can be a safety net.
            // For now, relying on the config being correctly set up.

            if let Some(url_to_add) = image_source_found {
                if !url_to_add.is_empty() && !image_urls.contains(&url_to_add) {
                    // Avoid duplicates URLs
                    image_urls.push(url_to_add)
                }
            } else {
                // Log if an image element was selected but no URL could be extracted
                // It might be useful to log element_img.html() here for debugging selectors
                eprintln!(
                    "[PARSER] Could not extract a valid image URL from element: {:?}",
                    img_element.value().name()
                );
            }
        }
    }
    println!("[PARSER] Found {} image URLs", image_urls.len());
    Ok(image_urls)
}

/// Extracts information (title, URL, number) from all chapters on the series main page.
/// Uses site configuration to determine selectors and how to parse chapter numbers.
/// Note: Regex compilation can be a performance bottleneck if called very frequently for many series.
/// Consider pre-compiling regexes or using once_cell::sync::Lazy if this becomes an issue.
pub async fn extract_chapter_links(
    series_page_html: &str,      // HTML content of the series main page
    series_page_url: &str,       // URL of the series main page (for absolutifying relative links)
    config: &SiteScrapingConfig, // Scraping configuration for this site
) -> Result<Vec<ChapterInfo>> {
    println!(
        "[PARSER] Parsing series page HTML for chapter links using selector: '{}'",
        config.chapter_link_selector
    );
    let document = Html::parse_document(series_page_html);

    let chapter_link_selector = Selector::parse(&config.chapter_link_selector).map_err(|e| {
        anyhow::anyhow!(
            "Invalid CSS selector for chapter links: '{}'. Error: {:?}",
            config.chapter_link_selector,
            e
        )
    })?;

    // Prepare regexes if they are defined in the configuration (optional)
    // These are compiled on each call. For high performance, consider compiling them once.
    let url_re = config
        .chapter_number_from_url_regex
        .as_ref()
        .and_then(|s| Regex::new(s).ok());
    let text_re = config
        .chapter_number_from_text_regex
        .as_ref()
        .and_then(|s| Regex::new(s).ok());

    let mut chapter_infos = Vec::new();

    for link_element in document.select(&chapter_link_selector) {
        // Should be <a> tags
        if let Some(href) = link_element.value().attr("href") {
            // Get href attribute (chapter URL)
            let trimmed_href = href.trim();
            if !trimmed_href.is_empty() {
                // Create absolute URL from href if it's relative
                match utils::to_absolute_url(series_page_url, trimmed_href) {
                    Ok(abs_url) => {
                        let title = link_element.text().collect::<String>().trim().to_string(); // Get text from link as title
                        let mut chapter_number_candidate: Option<f32> = None;

                        // Strategy for Extracting Chapter Number (with priority):
                        // 1. From data attribute on a parent element (e.g., <li data-chapterno="X">)
                        if chapter_number_candidate.is_none() {
                            if let Some(attr_name) = &config.chapter_number_data_attribute_on_parent
                            {
                                // Try to find the attribute on the link_element itself or its parents
                                let mut current_element_for_attr = Some(link_element);
                                while let Some(el) = current_element_for_attr {
                                    if let Some(data_no_str) = el.value().attr(attr_name) {
                                        if let Ok(num) = data_no_str.trim().parse::<f32>() {
                                            if num > 0.0 {
                                                // Basic validation
                                                chapter_number_candidate = Some(num);
                                                println!(
                                                    "[PARSER] Chapter number for '{}' from parent attribute '{}': {}",
                                                    title, attr_name, num
                                                );
                                                break;
                                            }
                                        }
                                    }
                                    current_element_for_attr = el.parent_element();
                                }
                            }
                        }

                        // 2. From chapter URL using regex (if not found yet)
                        if chapter_number_candidate.is_none() {
                            if let Some(re) = &url_re {
                                if let Some(caps) = re.captures(&abs_url) {
                                    if let Some(num_match) = caps.get(1) {
                                        // First capture group from regex
                                        if let Ok(num) = num_match.as_str().trim().parse::<f32>() {
                                            if num > 0.0 {
                                                // Basic validation
                                                chapter_number_candidate = Some(num);
                                                println!(
                                                    "[PARSER] Chapter number for '{}' from link text: {}",
                                                    title, num
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 3. From link text using regex (if not found yet)
                        if chapter_number_candidate.is_none() {
                            if let Some(re) = &text_re {
                                if let Some(caps) = re.captures(&title) {
                                    if let Some(num_match) = caps.get(1) {
                                        // Grup capture pertama dari regex
                                        if let Ok(num) = num_match.as_str().trim().parse::<f32>() {
                                            if num > 0.0 {
                                                chapter_number_candidate = Some(num);
                                                println!(
                                                    "[PARSER] Nomor chapter '{}' dari teks link: {}",
                                                    title, num
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // If chapter number was successfully extracted and is valid
                        if let Some(number) = chapter_number_candidate {
                            if !chapter_infos
                                .iter()
                                .any(|ci: &ChapterInfo| ci.number == number && ci.url == abs_url)
                            {
                                // Avoid duplicates
                                chapter_infos.push(ChapterInfo {
                                    title,
                                    url: abs_url,
                                    number,
                                });
                            } else {
                                println!(
                                    "[PARSER] Gagal parse nomor chapter yang valid untuk: '{}' (URL: {}). Dilewati.",
                                    title, abs_url
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "[PARSER] Failed to create absolute URL for chapter link '{}': {}. Skipping.",
                            trimmed_href, e
                        );
                    }
                }
            }
        }
    }

    // Urutkan chapter berdasarkan nomornya (dari kecil ke besar)
    chapter_infos.sort_by(|a, b| {
        a.number
            .partial_cmp(&b.number)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    println!(
        "[PARSER] Ditemukan {} link chapter valid setelah parsing.",
        chapter_infos.len()
    );
    Ok(chapter_infos)
}

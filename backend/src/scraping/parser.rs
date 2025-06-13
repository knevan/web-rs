use crate::core::utils;
use crate::scraping::model::SiteScrapingConfig;
use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};

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

    let mut image_urls = Vec::new(); // Vector to hold data

    for img_element in document.select(&image_element_selector) {
        let mut image_source_found: Option<String> = None;

        // 1. Try the primary attribute specified in config (e.g., src)
        if let Some(src_val) = img_element.value().attr(&config.image_url_attribute) {
            let trimmed_src = src_val.trim();
            if !trimmed_src.is_empty() {
                if let Ok(abs_url) = utils::to_absolute_url(base_url_relative_path, trimmed_src) {
                    println!(
                        "[PARSER] Found image URL from primary attribute '{}': {}",
                        config.image_url_attribute, abs_url
                    );
                    image_source_found = Some(abs_url);
                }
            }
        }

        // 2. If not found, try fallback attributes from config (e.g., data-src, data-lazy-src, etc.)
        if image_source_found.is_none() {
            for fallback_attr in &config.image_url_fallback_attributes {
                if let Some(src_val) = img_element.value().attr(fallback_attr) {
                    let trimmed_src = src_val.trim();
                    if !trimmed_src.is_empty() {
                        if let Ok(abs_url) =
                            utils::to_absolute_url(base_url_relative_path, trimmed_src)
                        {
                            println!(
                                "[PARSER] Found image URL from fallback attribute '{}': {}",
                                fallback_attr, abs_url
                            );
                            image_source_found = Some(abs_url);
                            break; // Stop fallback loop if found a valid URL
                        }
                    }
                }
            }
        }

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

    if image_urls.is_empty() {
        println!("[PARSER] Finished parsing, but no valid url were collected");
    } else {
        println!("[PARSER] Found {} unique image URLs", image_urls.len());
    }
    Ok(image_urls)
}

/// Extracts information (title, URL, number) from all chapters on the series main page.
/// Uses site configuration to determine selectors and how to parse chapter numbers.
/// [NOTE]: Regex compilation can be a performance bottleneck if called very frequently for many series.
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
                        // 1. From chapter URL using regex
                        if chapter_number_candidate.is_none() {
                            if let Some(re) = &url_re {
                                if let Some(caps) = re.captures(&abs_url) {
                                    if let Some(num_match) = caps.get(1) {
                                        // First capture group from regex
                                        if let Ok(num) = num_match
                                            .as_str()
                                            .trim()
                                            .replace('_', ".")
                                            .parse::<f32>()
                                        {
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

                        // 2. From link text using regex (if not found from url)
                        if chapter_number_candidate.is_none() {
                            if let Some(re) = &text_re {
                                if let Some(caps) = re.captures(&title) {
                                    if let Some(num_match) = caps.get(1) {
                                        // Grup capture pertama dari regex
                                        if let Ok(num) = num_match
                                            .as_str()
                                            .trim()
                                            .replace('_', ".")
                                            .parse::<f32>()
                                        {
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

                        // 3. Try from data attribute on a parent element (as a fallback)
                        if chapter_number_candidate.is_none() {
                            if let Some(attr_name) = &config.chapter_number_data_attribute_on_parent
                            {
                                if !attr_name.trim().is_empty() {
                                    // Only proceed if attribute name is specified
                                    let current_element_for_attr = Some(link_element);
                                    while let Some(el) = current_element_for_attr {
                                        if let Some(data_no_attr) = el.value().attr(attr_name) {
                                            if let Ok(num) =
                                                data_no_attr.trim().replace('_', ".").parse::<f32>()
                                            {
                                                if num > 0.0 {
                                                    chapter_number_candidate = Some(num);
                                                    println!(
                                                        "[PARSER] Chapter number for '{}' from parent attribute '{}': {}",
                                                        title, attr_name, num
                                                    );
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // If chapter number was successfully extracted and is valid
                        if let Some(number) = chapter_number_candidate {
                            // Check for duplicates based on chapter number AND URL to be safer
                            if !chapter_infos
                                .iter()
                                .any(|ci: &ChapterInfo| ci.number == number && ci.url == abs_url)
                            {
                                // Avoid duplicates
                                chapter_infos.push(ChapterInfo {
                                    title: title.clone(),
                                    url: abs_url.clone(), // Clone abs_url as it's used in logging too
                                    number,
                                });
                                println!(
                                    "[PARSER] Extracted Chapter: '{}', Number: {:.1}, URL: {}",
                                    title, number, abs_url
                                );
                            } else {
                                println!(
                                    "[PARSER] Duplicate chapter found (number: {:.1}, URL: {}). Skipping.",
                                    number, abs_url
                                );
                            }
                        } else {
                            println!(
                                "[PARSER] Failed to parse a valid chapter number for: '{}' (URL: {}). Skipping.",
                                title, abs_url
                            );
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

    if !chapter_infos.is_empty() {
        println!(
            "[PARSER] Ditemukan {} link chapter valid setelah parsing.",
            chapter_infos.len() // Optional: Log first and last chapter found to verify sorting
                                // if let Some(first) = chapter_infos.first() { println!("[PARSER] First chapter after sort: No. {:.1}", first.number); }
                                // if let Some(last) = chapter_infos.last() { println!("[PARSER] Last chapter after sort: No. {:.1}", last.number); }
        );
    } else {
        println!("[PARSER] No chapter links found after parsing.");
    }
    Ok(chapter_infos)
}

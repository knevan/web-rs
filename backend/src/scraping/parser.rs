use std::collections::HashMap;

use anyhow::Result;
use regex::Regex;
use scraper::{Element, ElementRef, Html, Selector};
use url::Url;

use crate::common::utils;
use crate::scraping::model::SiteScrapingConfig;

#[derive(Debug, Clone)]
pub struct ChapterInfo {
    pub url: String,
    pub number: f32,
}

// This struct pre-compiled selectors and regexes
pub struct ChapterParser {
    config: SiteScrapingConfig,
    chapter_link_selector: Selector,
    url_re: Option<Regex>,
    text_re: Option<Regex>,
}

impl ChapterParser {
    // Creates a new parser instance with compiled configurations
    pub fn new(config: SiteScrapingConfig) -> Result<Self> {
        let chapter_link_selector =
            Selector::parse(&config.chapter_link_selector).map_err(|e| {
                anyhow::anyhow!(
                    "Invalid CSS selector for image {}: {:?}",
                    &config.image_selector_on_chapter_page,
                    e
                )
            })?;

        // Pre-compile regexes and store
        let url_re = config
            .chapter_number_from_url_regex
            .as_deref()
            .and_then(|s| Regex::new(s).ok());

        let text_re = config
            .chapter_number_from_text_regex
            .as_deref()
            .and_then(|s| Regex::new(s).ok());

        Ok(Self {
            config,
            chapter_link_selector,
            url_re,
            text_re,
        })
    }

    // Helper to process a single link element into ChapterInfo
    fn process_link_element(
        &self,
        link_element: ElementRef,
        series_page_url: &str,
    ) -> Result<Option<ChapterInfo>> {
        if let Some(href) = link_element.value().attr("href") {
            let trimmed_href = href.trim();
            if trimmed_href.is_empty() {
                return Ok(None);
            }

            let abs_url = utils::to_absolute_url(series_page_url, trimmed_href)?;
            let title = link_element.text().collect::<String>().trim().to_string();

            // Find the chapter number using a prioritized strategy
            if let Some(number) =
                self.find_chapter_number_with_strategies(link_element, &abs_url, &title)
            {
                return Ok(Some(ChapterInfo {
                    url: abs_url,
                    number,
                }));
            }
        }
        Ok(None)
    }

    // Helper function to extract number from regex match
    fn extract_number_from_regex(&self, regex: &Regex, input: &str) -> Option<f32> {
        regex
            .captures(input)
            .and_then(|captures| captures.get(1))
            .and_then(|match_result| {
                // Captured string can be "-1" or ".1"
                let num_str = match_result.as_str();
                let replaced_str = num_str.replace('-', ".");

                replaced_str.parse::<f32>().ok()
            })
    }

    // Extracts a chapter number by trying multiple strategies
    fn find_chapter_number_with_strategies(
        &self,
        element: ElementRef,
        url: &str,
        title: &str,
    ) -> Option<f32> {
        // Helper closure to parse a string capture into f32
        let parse_match = |s: &str| s.parse::<f32>().ok();

        if let Some(attr_name) = &self.config.chapter_number_data_attribute_on_parent
            && !attr_name.is_empty()
        {
            // Simple loop on `parent_element`
            let mut current_element = Some(element);
            while let Some(el) = current_element {
                if let Some(num_str) = el.value().attr(attr_name)
                    && let Some(num) = parse_match(num_str)
                {
                    return Some(num);
                }
                current_element = el.parent_element();
            }
        }

        // Try URL regex strategy
        if let Some(re) = &self.url_re
            && let Some(num) = self.extract_number_from_regex(re, url)
        {
            return Some(num);
        }

        // Try text regex strategy
        if let Some(regex) = &self.text_re
            && let Some(number) = self.extract_number_from_regex(regex, title)
        {
            return Some(number);
        }

        None
    }

    // Quick check to extracts only the latest chapter based on the configured chapter_order
    pub fn quick_check_extract_latest_chapter_info(
        &self,
        series_page_html: &str,
        series_page_url: &str,
    ) -> Result<Option<ChapterInfo>> {
        let document = Html::parse_document(series_page_html);

        // Select first or last element depending on configured site chapter_order
        let latest_chapter_element = if self.config.chapter_order.eq_ignore_ascii_case("asc") {
            document.select(&self.chapter_link_selector).next_back()
        } else {
            document.select(&self.chapter_link_selector).next()
        };

        if let Some(element) = latest_chapter_element {
            return self.process_link_element(element, series_page_url);
        }

        Ok(None)
    }

    // Helper function to count check number of chapter links on a page
    pub fn count_chapter_links(&self, series_page_html: &str) -> Result<usize> {
        let document = Html::parse_document(series_page_html);
        let count = document.select(&self.chapter_link_selector).count();
        Ok(count)
    }

    // Full scan to extract all chapters, ensure uniqueness and sorted order
    pub fn full_scan_extract_all_chapter_info(
        &self,
        series_page_html: &str,
        series_page_url: &str,
    ) -> Result<Vec<ChapterInfo>> {
        println!("[FULL SCAN] Parsing all chapter links");
        let document = Html::parse_document(series_page_html);

        // The key is the chapter number scaled to an integer to avoid float precision issues as HashMap keys.
        let mut chapter_map: HashMap<i32, ChapterInfo> = HashMap::new();

        for link_element in document.select(&self.chapter_link_selector) {
            if let Some(info) = self.process_link_element(link_element, series_page_url)? {
                let key = (info.number * 100.0) as i32;
                chapter_map.entry(key).or_insert(info);
            }
        }

        // Collect unique chapters from the map
        let mut chapters: Vec<ChapterInfo> = chapter_map.into_values().collect();

        // Sort chapters by their number to ensure correct processing order
        chapters.sort_by(|a, b| a.number.total_cmp(&b.number));

        println!("[FULL SCAN] Found {} unique chapters", chapters.len());
        Ok(chapters)
    }
}

pub fn extract_image_urls_from_html_content(
    html_content: &str,
    base_chapter_url_relative_path: &str,
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
                "Invalid CSS selector for image {}: {:?}",
                &config.image_selector_on_chapter_page,
                e
            )
        })?;

    let mut image_urls = Vec::new();

    // Helper function to reduce code duplication for URL resolution
    let try_resolve_url = |attr_val: Option<&str>| -> Option<String> {
        let trimmed_src = attr_val?.trim();
        if trimmed_src.is_empty() {
            return None;
        }
        // Try to resolve as a relative URL first, then fall back to parsing as is
        utils::to_absolute_url(base_chapter_url_relative_path, trimmed_src)
            .or_else(|_err| Url::parse(trimmed_src).map(|u| u.to_string()))
            .ok()
    };

    for img_element in document.select(&image_element_selector) {
        // Try primary attribute first
        let maybe_url = try_resolve_url(img_element.value().attr(&config.image_url_attribute))
            // If primary fail, iterate through other fallback and use the one that works
            .or_else(|| {
                config
                    .image_url_fallback_attributes
                    .iter()
                    .find_map(|attr| try_resolve_url(img_element.value().attr(attr)))
            });

        if let Some(url_to_add) = maybe_url {
            // Ensure no duplicate URLs are added. For images, order is important
            if !image_urls.contains(&url_to_add) {
                image_urls.push(url_to_add);
            }
        } else {
            eprintln!(
                "[PARSER] Could not extract a valid image URL from element: {:?}",
                img_element.html()
            );
        }
    }

    println!("[PARSER] Found {} image URLs", image_urls.len());
    Ok(image_urls)
}

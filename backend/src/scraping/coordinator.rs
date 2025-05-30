use crate::core::utils::download_and_save_image;
use crate::db::db::ManhwaSeries;
use crate::scraping::{fetcher, parser};
use anyhow::{Context, Result};
use reqwest::Client;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Process scraping and downloading series chapters
pub async fn process_series_chapter(
    series_data: &ManhwaSeries,
    series_base_path: &Path,
    chapter_to_scrape: &[f32],
    http_client: &Client,
    img_css_selector: &str,
) -> Result<Option<f32>> {
    println!(
        "[COORDINATOR] Starting processing series {}",
        series_data.title
    );

    let series_url_from_db = match &series_data.current_source_url {
        Some(url) if !url.is_empty() => url,
        _ => {
            println!(
                "[COORDINATOR] Source URL not valid for series {}.",
                series_data.title
            );
            return Err(anyhow::anyhow!(
                "Source URL not valid for series {}.",
                series_data.title
            ));
        }
    };

    let mut last_successfully_downloaded_chapter_this_run: Option<f32> = None;

    for &chapter_number_float in chapter_to_scrape {
        let chapter_number_int = chapter_number_float as i32; // Untuk URL dan nama folder
        println!(
            "\n[COORDINATOR] Memproses Chapter {} untuk seri '{}'...",
            chapter_number_int, series_data.title
        );

        // Bentuk URL chapter dari URL seri di DB
        // Spesifik untuk mgeko.cc: https://www.mgeko.cc/manga/nama-manga/chapter-NOMOR/
        let chapter_url = format!("{}chapter-{}/", series_url_from_db, chapter_number_int);

        println!(
            "[COORDINATOR] Fetching HTML untuk Chapter {} dari {}",
            chapter_number_int, chapter_url
        );
        let html_content = match fetcher::fetch_html(&chapter_url).await {
            Ok(content) => content,
            Err(e) => {
                eprintln!(
                    "[COORDINATOR] Gagal fetch HTML untuk Chapter {}: {}. Melanjutkan.",
                    chapter_number_int, e
                );
                continue;
            }
        };
        // Jeda setelah fetch HTML halaman chapter
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Parse HTML untuk mendapatkan URL gambar
        let image_urls =
            match parser::extract_image_urls(&html_content, &chapter_url, img_css_selector) {
                Ok(urls) => {
                    if urls.is_empty() {
                        println!(
                            "[COORDINATOR] Tidak ada URL gambar ditemukan untuk Chapter {}.",
                            chapter_number_int
                        );
                        continue;
                    }
                    urls
                }
                Err(e) => {
                    eprintln!(
                        "[COORDINATOR] Gagal parse gambar untuk Chapter {}: {}. Melanjutkan.",
                        chapter_number_int, e
                    );
                    continue;
                }
            };

        // Buat folder chapter jika belum ada
        let chapter_folder_name = format!("Chapter_{}", chapter_number_int);
        let chapter_path = series_base_path.join(chapter_folder_name);
        if !chapter_path.exists() {
            fs::create_dir_all(&chapter_path).with_context(|| {
                format!(
                    "Gagal membuat folder untuk Chapter {}: {:?}",
                    chapter_number_int, chapter_path
                )
            })?;
            println!(
                "[COORDINATOR] Folder Chapter {} dibuat di: {:?}",
                chapter_number_int, chapter_path
            );
        }

        // Download dan simpan setiap gambar
        println!(
            "[COORDINATOR] Mengunduh {} gambar untuk Chapter {}...",
            image_urls.len(),
            chapter_number_int
        );
        let mut images_downloaded_in_chapter_count = 0;
        for (index, img_url) in image_urls.iter().enumerate() {
            let extension = Path::new(img_url)
                .extension()
                .and_then(|os_str| os_str.to_str())
                .map_or("jpg", |ext| if ext.is_empty() { "jpg" } else { ext }); // Default ke jpg jika ekstensi kosong

            let image_filename = format!("{:03}.{}", index + 1, extension);
            let image_save_path = chapter_path.join(&image_filename);

            if let Err(e) = download_and_save_image(http_client, img_url, &image_save_path).await {
                eprintln!(
                    "[COORDINATOR] Gagal mengunduh/menyimpan gambar {} (Chapter {}): {}",
                    img_url, chapter_number_int, e
                );
            } else {
                images_downloaded_in_chapter_count += 1;
            }
        }

        if images_downloaded_in_chapter_count > 0 {
            last_successfully_downloaded_chapter_this_run = Some(chapter_number_float);
        }
        println!(
            "[COORDINATOR] Selesai memproses Chapter {}. {} gambar diunduh.",
            chapter_number_int, images_downloaded_in_chapter_count
        );

        // Jeda antar chapter
        println!("[COORDINATOR] Jeda sebelum chapter berikutnya (jika ada)...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    Ok(last_successfully_downloaded_chapter_this_run)
}

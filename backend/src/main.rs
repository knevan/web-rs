//mod processing;
mod core;
mod db;
mod scraping;

use anyhow::{Context, Result};
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("[MAIN] App starting...");

    let current_dir = current_dir().with_context(|| "Failed to get current directory")?;
    println!("[MAIN] Current working directory: {:?}", current_dir); // Tambahkan baris ini

    let db_path = "manhwa_list.sqlite3";
    let db_sql_file = "backend/src/db/db.sql";
    let root_data_dir = PathBuf::from("downloaded_manhwa");

    if !root_data_dir.exists() {
        fs::create_dir_all(&root_data_dir).with_context(|| {
            format!(
                "Failed to create root directory: {}",
                root_data_dir.display()
            )
        })?;
        println!("[MAIN] Root directory created: {:?}", root_data_dir);
    }

    println!("[MAIN] Conneting to database: {}", db_path);
    let conn = db::db::connect_db(db_path)
        .with_context(|| format!("Failed to connect to database: {}", db_path))?;

    db::db::initialize_schema(&conn, db_sql_file)
        .with_context(|| format!("Failed to initialize database schema from {}", db_sql_file))?;
    println!("[MAIN] Database and Schema initialized");

    // Initialize HTTP Client
    let http_client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.4844.51 Safari/537.36 RustScraper/0.1")
        .build()?;
    println!("[MAIN] HTTP Client berhasil dibuat.");

    // --- 4. Logika Pemilihan Seri untuk Diproses (Contoh: Satu Seri Target untuk Tes) ---
    let target_series_title = "Limit-Breaking Genius Mage";
    let target_series_url_default = "https://www.mgeko.cc/manga/limit-breaking-genius-mage/";
    let default_check_interval: i32 = 120; // menit
    let image_css_selector = "div.reading-content img.wp-manga-chapter-img"; // Selector untuk mgeko.cc

    // Dapatkan atau buat entri seri di database
    let series_data = match db::db::get_manhwa_series_by_title(&conn, target_series_title)? {
        Some(mut series) => {
            println!(
                "[MAIN] Series '{}' found in database (ID: {}).",
                series.title, series.id
            );
            if series
                .current_source_url
                .clone()
                .is_none_or(|s| s.trim().is_empty() || s != target_series_url_default)
            {
                println!(
                    "[MAIN] URL sumber untuk '{}' berbeda atau kosong di DB, mengupdate dengan URL default: {}",
                    series.title, target_series_url_default
                );
                db::db::update_series_source_urls(&conn, series.id, target_series_url_default)?;
                // Update data series lokal setelah perubahan DB
                series.current_source_url = Some(target_series_url_default.to_string());
                series.source_website_host = url::Url::parse(target_series_url_default)
                    .ok()
                    .and_then(|u| u.host_str().map(String::from));
            }
            series
        }
        None => {
            println!(
                "[MAIN] Seri '{}' tidak ditemukan di database. Menambahkan...",
                target_series_title
            );
            let new_series_id = db::db::add_manhwa_series(
                &conn,
                target_series_title,
                Some(target_series_url_default),
                default_check_interval,
            )?;
            println!(
                "[MAIN] Seri '{}' berhasil ditambahkan dengan ID: {}.",
                target_series_title, new_series_id
            );
            db::db::get_manhwa_series_by_id(&conn, new_series_id as i32)?
                .expect("Seri yang baru ditambahkan seharusnya ada")
        }
    };

    // Persiapan path folder lokal untuk seri ini
    // Pastikan `core::utils::sanitize_series_title` ada dan benar
    let series_folder_name = core::utils::sanitize_series_title(&series_data.title);
    let series_base_path = root_data_dir.join(&series_folder_name);

    if !series_base_path.exists() {
        fs::create_dir_all(&series_base_path)
            .with_context(|| format!("Gagal membuat folder seri: {:?}", series_base_path))?;
        println!("[MAIN] Folder seri dibuat: {:?}", series_base_path);
    }

    // Tentukan chapter yang akan di-scrape untuk tes ini
    // Untuk pengujian, kita scrape chapter 1 dan 2
    // Dalam implementasi penuh, Anda akan menentukan ini berdasarkan `series_data.last_chapter_found_locally`
    // dan informasi dari halaman list chapter di website sumber.
    let chapters_to_scrape_for_this_test: [f32; 2] = [1.0, 2.0];
    // Contoh logika lanjutan:
    // let start_chapter = series_data.last_chapter_found_locally.unwrap_or(0.0) + 1.0;
    // let chapters_to_scrape_for_this_test: Vec<f32> = (0..2).map(|i| start_chapter + i as f32).collect();

    println!(
        "[MAIN] Memulai proses scraping untuk seri: '{}'",
        series_data.title
    );
    match scraping::coordinator::process_series_chapter(
        // Ganti nama fungsi jika berbeda di coordinator.rs Anda
        &series_data,
        &series_base_path,
        &chapters_to_scrape_for_this_test,
        &http_client,
        image_css_selector,
    )
    .await
    {
        Ok(Some(last_downloaded_chapter)) => {
            println!(
                "[MAIN] Scraping batch selesai. Chapter terakhir yang diunduh sesi ini: {}",
                last_downloaded_chapter
            );
            let current_local_chap_in_db = series_data.last_chapter_found_locally.unwrap_or(0.0);
            if last_downloaded_chapter > current_local_chap_in_db {
                println!(
                    "[MAIN] Mengupdate last_chapter_found_locally di DB untuk seri '{}' menjadi {}.",
                    series_data.title, last_downloaded_chapter
                );
                db::db::update_series_last_local_chapter(
                    &conn,
                    series_data.id,
                    Some(last_downloaded_chapter),
                )?;
            }
        }
        Ok(None) => {
            println!(
                "[MAIN] Scraping batch selesai. Tidak ada chapter baru yang berhasil diunduh dalam sesi ini."
            );
        }
        Err(e) => {
            eprintln!(
                "[MAIN] Terjadi error selama proses scraping untuk seri '{}': {}",
                series_data.title, e
            );
            // Pertimbangkan untuk mengupdate status seri di DB menjadi 'error' di sini
            // db::update_series_processing_status(&conn, series_data.id, "error")?;
        }
    }

    // Update jadwal pengecekan di database
    let current_ts = db::db::current_timestamp();
    db::db::update_series_check_schedule(
        &conn,
        series_data.id,
        Some("monitoring"),
        Some(current_ts),
        None,
    )?;
    println!(
        "[MAIN] Jadwal pengecekan untuk seri '{}' telah diupdate.",
        series_data.title
    );

    println!("\n[MAIN] Aplikasi selesai.");
    Ok(())
}

/*let jpg_input_folder: &str = "imgjpg";
let jpg_output_folder: &str = "outputjpg";
let png_input_folder: &str = "imgpng";
let png_output_folder: &str = "outputpng";

println!("=== Program Konversi Gambar ke AVIF ===\n");

if !Path::new(jpg_input_folder).exists() {
    eprintln!(
        "Error: File input '{}' tidak ditemukan. Harap sediakan file gambar.",
        jpg_input_folder
    );
} else {
    process_folder(jpg_input_folder, jpg_output_folder).await?;
}

if !Path::new(png_input_folder).exists() {
    eprintln!(
        "Error: File input '{}' tidak ditemukan. Harap sediakan file gambar.",
        png_input_folder
    );
} else {
    process_folder(png_input_folder, png_output_folder).await?;
}

println!("Semua proses konversi selesai!");
Ok(())*/

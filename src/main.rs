use image::GenericImageView;
use ravif::{Encoder, Img, RGB8, RGBA8};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;
use tokio::{fs, task};

struct MemoryStats {
    samples: Vec<u64>,
    peak: u64,
}

impl MemoryStats {
    fn new() -> Self {
        MemoryStats {
            samples: Vec::new(),
            peak: 0,
        }
    }

    fn add_sample(&mut self, memory_usage: u64) {
        self.samples.push(memory_usage);
        if memory_usage > self.peak {
            self.peak = memory_usage;
        }
    }

    fn average(&self) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        self.samples.iter().sum::<u64>() / self.samples.len() as u64
    }
}

fn get_current_memory_usage() -> Result<u64, Box<dyn std::error::Error>> {
    let mut system = System::new_all();
    system.refresh_all();

    let pid = std::process::id() as usize;

    if let Some(process) = system.process(sysinfo::Pid::from(pid)) {
        Ok(process.memory() * 1024) // sysinfo returns memory in KB, so multiply by 1024 to get bytes
    } else {
        Err("Could not find process".into())
    }
}

async fn covert_to_avif(
    input_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let input_size = fs::metadata(input_path).await?.len();

    let img = task::spawn_blocking({
        let input_path_owned = input_path.to_owned();
        move || image::open(input_path_owned)
    })
    .await??;

    let (width, height) = img.dimensions();

    let encode = Encoder::new()
        .with_quality(40.0)
        .with_alpha_quality(40.0)
        .with_speed(5);

    let avif_data = task::spawn_blocking(move || {
        if img.color().has_alpha() {
            let rgba_image = img.to_rgba8();

            // Kumpulkan piksel menjadi Vec<RGBA8>
            // Perhatikan bahwa `image::RgbaImage::pixels()` mengembalikan iterator
            // dari `Rgba<u8>` (tipe piksel dari 'image' crate).
            // Kita perlu mengubahnya menjadi `ravif::RGBA8`.
            let rgba_pixels: Vec<RGBA8> = rgba_image
                .pixels()
                .map(|p| RGBA8 {
                    r: p[0],
                    g: p[1],
                    b: p[2],
                    a: p[3],
                })
                .collect();

            // Sekarang kita bisa memberikan slice dari Vec<RGBA8> ke Img::new
            encode.encode_rgba(Img::new(
                rgba_pixels.as_slice(), // Ini adalah &[RGBA8]
                width as usize,
                height as usize,
            ))
        } else {
            let rgb_image = img.to_rgb8(); // Konversi ke RGB8 buffer

            // Kumpulkan piksel menjadi Vec<RGB8>
            // Seperti di atas, `image::RgbImage::pixels()` mengembalikan iterator
            // dari `Rgb<u8>` (tipe piksel dari 'image' crate).
            // Kita perlu mengubahnya menjadi `ravif::RGB8`.
            let rgb_pixels: Vec<RGB8> = rgb_image
                .pixels()
                .map(|p| RGB8 {
                    r: p[0],
                    g: p[1],
                    b: p[2],
                })
                .collect();

            // Sekarang kita bisa memberikan slice dari Vec<RGB8> ke Img::new
            encode.encode_rgb(Img::new(
                rgb_pixels.as_slice(), // Ini adalah &[RGB8]
                width as usize,
                height as usize,
            ))
        }
    })
    .await??;

    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent).await?;
    }

    fs::write(output_path, avif_data.avif_file).await?;

    let output_size = fs::metadata(output_path).await?.len();

    let compression_ratio = if input_size > 0 {
        (1.0 - (output_size as f64 / input_size as f64)) * 100.0
    } else {
        0.0
    };

    let duration = start_time.elapsed();

    println!("Waktu konversi: {:.2?}", duration);
    println!("Konversi berhasil: {} -> {}", input_path, output_path);
    println!("Ukuran file input: {} bytes", input_size);
    println!("Ukuran file output: {} bytes", output_size);
    println!("Kompresi: {:.2}% pengurangan", compression_ratio);

    Ok(())
}

async fn process_folder(
    input_folder: &str,
    output_folder: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_start_time = Instant::now();

    // Gunakan flag untuk mengontrol thread pemantau memori
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let memory_monitor_handle = thread::spawn(move || {
        let mut local_memory_stats = MemoryStats::new();

        while running_clone.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));

            match get_current_memory_usage() {
                Ok(memory) => local_memory_stats.add_sample(memory),
                Err(_) => break,
            }
        }

        local_memory_stats
    });

    // Pastikan folder output ada
    fs::create_dir_all(output_folder).await?;

    // Baca semua file dalam folder input
    let mut entries = match fs::read_dir(input_folder).await {
        Ok(entries) => entries,
        Err(e) => {
            // Hentikan thread pemantau memori jika terjadi error
            running.store(false, Ordering::Relaxed);
            return Err(e.into());
        }
    };

    let mut total_files = 0;
    let mut processed_files = 0;
    let mut total_input_size = 0;
    let mut total_output_size = 0;

    // Hitung jumlah file terlebih dahulu
    let mut temp_entries_count = fs::read_dir(input_folder).await?;
    while let Some(entry_result) = temp_entries_count.next_entry().await? {
        let path = entry_result.path();

        if !path.is_file() {
            continue;
        }

        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        if ["jpg", "jpeg", "png"].contains(&extension.to_lowercase().as_str()) {
            total_files += 1;
            match fs::metadata(&path).await {
                Ok(metadata) => total_input_size += metadata.len(),
                Err(e) => eprintln!("Error saat membaca metadata {}: {}", path.display(), e),
            }
        }
    }

    // Proses setiap file
    //let mut current_entries = fs::read_dir(input_folder).await?;
    while let Some(entry_result) = entries.next_entry().await? {
        let entry = entry_result;

        let path = entry.path();

        // Lewati jika bukan file
        if !path.is_file() {
            continue;
        }

        // Dapatkan ekstensi file
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        // Proses hanya file gambar (jpg, jpeg, png)
        if ["jpg", "jpeg", "png"].contains(&extension.to_lowercase().as_str()) {
            // Buat path output
            let file_name = match path.file_name() {
                Some(name) => match name.to_str() {
                    Some(s) => s,
                    None => {
                        eprintln!("Error: Nama file tidak valid");
                        continue;
                    }
                },
                None => {
                    eprintln!("Error: Tidak dapat mendapatkan nama file");
                    continue;
                }
            };

            let base_name = file_name
                .rsplit_once('.')
                .map(|(name, _)| name)
                .unwrap_or(file_name);
            let output_path = PathBuf::from(output_folder).join(format!("{}.avif", base_name));

            println!(
                "Memproses: {} ({}/{})",
                path.display(),
                processed_files + 1,
                total_files
            );

            // Konversi gambar
            match covert_to_avif(path.to_str().unwrap(), output_path.to_str().unwrap()).await {
                Ok(_) => {
                    processed_files += 1;
                    match fs::metadata(&output_path).await {
                        Ok(metadata) => total_output_size += metadata.len(),
                        Err(e) => eprintln!(
                            "Error saat membaca metadata output {}: {}",
                            output_path.display(),
                            e
                        ),
                    }
                }
                Err(e) => eprintln!("Error saat memproses {}: {}", path.display(), e),
            }
        }
    }

    // Hentikan thread pemantau memori
    running.store(false, Ordering::Relaxed);

    // Tunggu thread pemantau memori selesai
    let memory_stats = memory_monitor_handle.join().unwrap_or_else(|_| {
        eprintln!("Error: Thread pemantau memori mengalami masalah");
        MemoryStats::new()
    });

    let total_duration = total_start_time.elapsed();

    let total_compression_ratio = if total_input_size > 0 {
        (1.0 - (total_output_size as f64 / total_input_size as f64)) * 100.0
    } else {
        0.0
    };

    println!("\n===== RINGKASAN KONVERSI FOLDER: {} =====", input_folder);
    println!("Total waktu konversi: {:.2?}", total_duration);
    println!("Total file diproses: {}/{}", processed_files, total_files);
    println!(
        "Total ukuran input: {:.2} MB",
        total_input_size as f64 / (1024.0 * 1024.0)
    );
    println!(
        "Total ukuran output: {:.2} MB",
        total_output_size as f64 / (1024.0 * 1024.0)
    );
    println!(
        "Kompresi rata-rata: {:.2}% pengurangan",
        total_compression_ratio
    );
    println!(
        "Penggunaan RAM puncak: {:.2} MB",
        memory_stats.peak as f64 / (1024.0 * 1024.0)
    );
    println!(
        "Penggunaan RAM rata-rata: {:.2} MB",
        memory_stats.average() as f64 / (1024.0 * 1024.0)
    );
    println!("===========================================\n");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jpg_input_folder: &str = "imgjpg";
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
    Ok(())
}

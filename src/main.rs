use std::fs;
use ravif::{Encoder, Img, RGB8, RGBA8};
use std::path::{Path, PathBuf};
use std::time::Instant;
use image::{GenericImageView};

fn covert_to_avif(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    let input_size = fs::metadata(input_path)?.len();

    let img = image::open(input_path)?;

    let (width, height) = img.dimensions();

    let encode = Encoder::new()
        .with_quality(40.0)
        .with_alpha_quality(40.0)
        .with_speed(5);

    let avif_data = if img.color().has_alpha() {
        let rgba_image = img.to_rgba8();

        // Kumpulkan piksel menjadi Vec<RGBA8>
        // Perhatikan bahwa `image::RgbaImage::pixels()` mengembalikan iterator
        // dari `Rgba<u8>` (tipe piksel dari 'image' crate).
        // Kita perlu mengubahnya menjadi `ravif::RGBA8`.
        let  rgba_pixels: Vec<RGBA8> = rgba_image.pixels().map(|p| RGBA8 {
            r: p[0],
            g: p[1],
            b: p[2],
            a: p[3],
        }).collect();

        // Sekarang kita bisa memberikan slice dari Vec<RGBA8> ke Img::new
        encode.encode_rgba(Img::new(
            rgba_pixels.as_slice(), // Ini adalah &[RGBA8]
            width as usize,
            height as usize,
        ))?
    } else {
        let rgb_image = img.to_rgb8(); // Konversi ke RGB8 buffer

        // Kumpulkan piksel menjadi Vec<RGB8>
        // Seperti di atas, `image::RgbImage::pixels()` mengembalikan iterator
        // dari `Rgb<u8>` (tipe piksel dari 'image' crate).
        // Kita perlu mengubahnya menjadi `ravif::RGB8`.
        let rgb_pixels: Vec<RGB8> = rgb_image.pixels().map(|p| RGB8 {
            r: p[0],
            g: p[1],
            b: p[2],
        }).collect();

        // Sekarang kita bisa memberikan slice dari Vec<RGB8> ke Img::new
        encode.encode_rgb(Img::new(
            rgb_pixels.as_slice(), // Ini adalah &[RGB8]
            width as usize,
            height as usize,
        ))?
    };

    fs::write(output_path, avif_data.avif_file)?;
    
    let  output_size = fs::metadata(output_path)?.len();
    
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

fn process_folder(input_folder: &str, output_folder: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Pastikan folder output ada
    fs::create_dir_all(output_folder)?;

    // Baca semua file dalam folder input
    let entries = fs::read_dir(input_folder)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Lewati jika bukan file
        if !path.is_file() {
            continue;
        }

        // Dapatkan ekstensi file
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // Proses hanya file gambar (jpg, jpeg, png)
        if ["jpg", "jpeg", "png"].contains(&extension.to_lowercase().as_str()) {
            // Buat path output
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let base_name = file_name.rsplit_once('.').map(|(name, _)| name).unwrap_or(file_name);
            let output_path = PathBuf::from(output_folder).join(format!("{}.avif", base_name));

            println!("Memproses: {}", path.display());

            // Konversi gambar
            if let Err(e) = covert_to_avif(
                path.to_str().unwrap(),
                output_path.to_str().unwrap()
            ) {
                eprintln!("Error saat memproses {}: {}", path.display(), e);
            }
        }
    }

    println!("Selesai memproses folder: {}", input_folder);
    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jpg_input_folder: &str = "imgjpg";
    let jpg_output_folder: &str = "outputjpg";
    let png_input_folder: &str = "imgpng";
    let png_output_folder: &str = "outputpng";

    if !Path::new(jpg_input_folder).exists() {
        eprintln!("Error: File input '{}' tidak ditemukan. Harap sediakan file gambar.", jpg_input_folder);
    } else {
        process_folder(jpg_input_folder, jpg_output_folder)?;
    }
    
    if !Path::new(png_input_folder).exists() {
        eprintln!("Error: File input '{}' tidak ditemukan. Harap sediakan file gambar.", png_input_folder);
    } else {
        process_folder(png_input_folder, png_output_folder)?;
    }

    Ok(())
}
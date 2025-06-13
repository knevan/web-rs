use anyhow::{Context, Result};
use image::{GenericImageView, load_from_memory};
use ravif::{Encoder, Img, RGB8, RGBA8};

/// Converts image data from bytes (e.g., PNG, JPG) into AVIF format bytes.
/// This function is CPU-intensive and is designed to be run in a blocking thread
/// using `tokio::task::spawn_blocking` to avoid blocking the async runtime.
pub fn covert_image_bytes_to_avif(image_bytes: &[u8]) -> Result<Vec<u8>> {
    // Decode the image from memory
    let img = load_from_memory(image_bytes).with_context(
        || "Failed to decode image from memory. The format may be unsupported or data corrupted",
    )?;

    let (width, height) = img.dimensions();

    // Configure the AVIF encoder.
    // Quality: 0-100 (higher is better quality, larger file).
    // Speed: 0-10 (higher is faster encoding, potentially lower quality/compression).
    // A quality of ~50 and speed of 6 is a good balance.
    let encoder = Encoder::new()
        .with_quality(45.0)
        .with_alpha_quality(45.0) // Quality for transparency channel
        .with_speed(6);

    // Encode the image based on whether it has an alpha channel or not.
    let avif_result = if img.color().has_alpha() {
        let rgba_image = img.to_rgba8();
        // Convert pixels to the format rafiv expects.
        let pixels: Vec<RGBA8> = rgba_image
            .pixels()
            .map(|p| RGBA8 {
                r: p[0],
                g: p[1],
                b: p[2],
                a: p[3],
            })
            .collect();
        encoder.encode_rgba(Img::new(pixels.as_slice(), width as usize, height as usize))
    } else {
        let rgb_image = img.to_rgb8();
        let pixels: Vec<RGB8> = rgb_image
            .pixels()
            .map(|p| RGB8 {
                r: p[0],
                g: p[1],
                b: p[2],
            })
            .collect();
        encoder.encode_rgb(Img::new(pixels.as_slice(), width as usize, height as usize))
    };

    let avif_data = avif_result.with_context(|| "Failed to encode image to AVIF")?;

    println!(
        "[IMAGE ENCODING] Successfully converted image ({}x{}) to AVIF format. Size: {} bytes",
        width,
        height,
        avif_data.avif_file.len()
    );

    Ok(avif_data.avif_file)
}

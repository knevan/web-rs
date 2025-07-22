use anyhow::{Context, Result};
use image::{GenericImageView, load_from_memory};
use ravif::{Encoder, Img};
use rgb::FromSlice;

/// Converts image data from bytes (e.g., PNG, JPG) into AVIF format bytes.
/// This function is CPU-intensive and is designed to be run in a blocking thread
/// [NOTE]: Using `tokio::task::spawn_blocking` to avoid blocking the async runtime in the future?
pub fn covert_image_bytes_to_avif(image_bytes: &[u8]) -> Result<Vec<u8>> {
    // Decode the image from memory
    let img = load_from_memory(image_bytes).with_context(
        || "Failed to decode image from memory. The format may be unsupported or data corrupted",
    )?;

    let (width_u32, height_u32) = img.dimensions();

    // Convert dimensions to usize, which is required by ravif's Img::new()
    let width = width_u32 as usize;
    let height = height_u32 as usize;

    // Quality: 0-100 (higher is better iamge quality, larger file size).
    // Speed: 0-10 (higher is faster encoding, lower quality/compression size).
    // A quality of ~40-50 and speed of ~5-6 is a good balance (good image quality and good compression size).
    let encoder = Encoder::new()
        .with_quality(45.0)
        .with_alpha_quality(45.0) // Quality for transparency channel
        .with_speed(6);

    // This is more efficient as it avoids manual iteration and reallocation.
    // Encode the image based on whether it has an alpha channel or not.
    let avif_result = if img.color().has_alpha() {
        let rgba_image = img.to_rgba8();
        // Use the FromSlice trait to perform a zero-cost cast from &[u8] to &[RGBA8]
        let pixels = rgba_image.as_rgba();
        encoder.encode_rgba(Img::new(pixels, width, height))
    } else {
        let rgb_image = img.to_rgb8();
        let pixels = rgb_image.as_rgb();
        encoder.encode_rgb(Img::new(pixels, width, height))
    };

    let avif_data =
        avif_result.with_context(|| "Failed to encode image to AVIF")?;

    println!(
        "[IMAGE ENCODING] Successfully converted image ({}x{}) to AVIF format. Size: {} bytes",
        width,
        height,
        avif_data.avif_file.len()
    );

    Ok(avif_data.avif_file)
}

use anyhow::{Context, Result};
use image::GenericImageView;
use ravif::{Encoder, Img, RGB8, RGBA8};
use tokio::task;

pub async fn covert_to_avif_in_memory(image_bytes: Vec<u8>) -> Result<Vec<u8>> {
    // Encoding is a CPU intensive operation.
    // Use `spawn_blocking` to run it in a separate thread pool so as not to block Tokio's async event loop.

    let avif_task = task::spawn_blocking(move || -> Result<Vec<u8>> {
        let img =
            image::load_from_memory(&image_bytes).context("Failed to decode image from memory")?;

        let (width, height) = img.dimensions();

        let encoder = Encoder::new()
            .with_quality(45.0)
            .with_alpha_quality(45.0)
            .with_speed(6);

        let result = if img.color().has_alpha() {
            let rgba_image = img.to_rgba8();
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

        match result {
            Ok(data) => Ok(data.avif_file),
            Err(e) => Err(anyhow::anyhow!("Failed to encode image to AVIF: {}", e)),
        }
    });

    avif_task.await?
}

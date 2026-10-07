use std::io::Cursor;

use image::{ImageFormat, ImageReader, Limits};
use sha2::{Digest, Sha256};

pub struct CanonicalImage {
    pub png: Vec<u8>,
    pub digest: String,
    pub width: u32,
    pub height: u32,
}

pub fn canonicalize(bytes: &[u8], target_long_edge: Option<u32>) -> Result<CanonicalImage, String> {
    if bytes.is_empty() || bytes.len() > 64 * 1024 * 1024 {
        return Err("Generated image must be non-empty and no larger than 64 MiB".into());
    }
    let mut reader = ImageReader::new(Cursor::new(bytes));
    reader = reader
        .with_guessed_format()
        .map_err(|error| error.to_string())?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(10_000);
    limits.max_image_height = Some(10_000);
    reader.limits(limits);
    let mut image = reader
        .decode()
        .map_err(|error| format!("Generated image could not be decoded: {error}"))?;
    if u64::from(image.width()) * u64::from(image.height()) > 100_000_000 {
        return Err("Generated image exceeds the pixel limit".into());
    }
    if let Some(target) = target_long_edge {
        let current = image.width().max(image.height());
        if current < target {
            return Err("Upscale provider returned an image below the requested resolution".into());
        }
        if current > target {
            image = image.resize(target, target, image::imageops::FilterType::Lanczos3);
        }
    }
    let (width, height) = (image.width(), image.height());
    let mut png = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .map_err(|error| format!("Generated image could not be encoded: {error}"))?;
    if png.len() > 64 * 1024 * 1024 {
        return Err("Generated PNG exceeds 64 MiB".into());
    }
    let digest = format!("sha256:{}", hex::encode(Sha256::digest(&png)));
    Ok(CanonicalImage {
        png,
        digest,
        width,
        height,
    })
}

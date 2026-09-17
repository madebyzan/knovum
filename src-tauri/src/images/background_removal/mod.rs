mod inference;
mod pixels;

#[cfg(test)]
mod tests;

use std::{fs, io::Cursor, path::Path};

use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, Limits};

use super::BackgroundRemovalOutput;

const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_IMAGE_PIXELS: u64 = 100_000_000;
const MAX_IMAGE_DIMENSION: u32 = 10_000;
const MAX_OUTPUT_BYTES: usize = 64 * 1024 * 1024;

pub(crate) fn remove_background(source: &Path) -> Result<BackgroundRemovalOutput, String> {
    let metadata = fs::metadata(source).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_SOURCE_BYTES {
        return Err("Background-removal input must be a non-empty image up to 64 MiB".into());
    }
    let mut reader = ImageReader::open(source)
        .map_err(|error| error.to_string())?
        .with_guessed_format()
        .map_err(|error| error.to_string())?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    reader.limits(limits);
    let image = reader.decode().map_err(|error| error.to_string())?;
    validate_dimensions(image.dimensions())?;
    let input = pixels::model_input(&image);
    let mask = inference::run(input)?;
    let foreground = pixels::apply_mask(image, &mask)?;
    let (width, height) = foreground.dimensions();
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(foreground)
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    let png = output.into_inner();
    if png.len() > MAX_OUTPUT_BYTES {
        return Err("Background-removal PNG exceeds 64 MiB".into());
    }
    Ok(BackgroundRemovalOutput { png, width, height })
}

fn validate_dimensions((width, height): (u32, u32)) -> Result<(), String> {
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_IMAGE_PIXELS {
        Err("Background-removal image dimensions are invalid".into())
    } else {
        Ok(())
    }
}

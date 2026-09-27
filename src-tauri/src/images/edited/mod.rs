mod contract;
mod eraser;
mod geometry;
mod glow;
mod pixels;
mod texture;
mod tone;

use std::{fs, io::Cursor, path::Path};

use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, Limits};
use serde_json::Value;

use self::contract::ImageEdit;
use super::EditedImageOutput;

const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SOURCE_PIXELS: u64 = 100_000_000;
const MAX_SOURCE_DIMENSION: u32 = 10_000;
const MAX_OUTPUT_BYTES: usize = 64 * 1024 * 1024;

pub(crate) fn validate_edit(value: &Value) -> Result<(), String> {
    ImageEdit::parse(value).map(|_| ())
}

/// The caller owns authorization; the engine enforces image resource limits.
pub(crate) fn render_source_png(source: &Path, edit: &Value) -> Result<EditedImageOutput, String> {
    let metadata = fs::metadata(source).map_err(|error| error.to_string())?;
    if metadata.len() == 0 || metadata.len() > MAX_SOURCE_BYTES {
        return Err("Edited-image input must be a non-empty image up to 64 MiB".into());
    }
    let edit = ImageEdit::parse(edit)?;
    let mut reader = ImageReader::open(source).map_err(|error| error.to_string())?;
    reader = reader
        .with_guessed_format()
        .map_err(|error| error.to_string())?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SOURCE_DIMENSION);
    limits.max_image_height = Some(MAX_SOURCE_DIMENSION);
    reader.limits(limits);
    let source = reader.decode().map_err(|error| error.to_string())?;
    validate_source_dimensions(source.dimensions())?;
    encode(pixels::render(&source.to_rgba8(), &edit)?)
}

fn encode(image: image::RgbaImage) -> Result<EditedImageOutput, String> {
    let (width, height) = image.dimensions();
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image)
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    let png = output.into_inner();
    if png.len() > MAX_OUTPUT_BYTES {
        return Err("Edited-image PNG exceeds 64 MiB".into());
    }
    Ok(EditedImageOutput { png, width, height })
}

fn validate_source_dimensions((width, height): (u32, u32)) -> Result<(), String> {
    if width == 0
        || height == 0
        || width > MAX_SOURCE_DIMENSION
        || height > MAX_SOURCE_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_SOURCE_PIXELS
    {
        Err("Edited-image source dimensions are invalid".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;

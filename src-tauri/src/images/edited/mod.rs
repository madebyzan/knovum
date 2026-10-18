mod contracts;
mod geometry;
mod glow;
mod pixels;
mod texture;
mod tone;

use std::{fs, io::Cursor, path::Path};

use image::{GenericImageView, ImageEncoder, ImageFormat, ImageReader, Limits};
use serde_json::Value;

use self::contracts::ImageEdit;
use super::EditedImageOutput;

const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SOURCE_PIXELS: u64 = 100_000_000;
const MAX_SOURCE_DIMENSION: u32 = 10_000;
/// Below this edge a downscaled reference stops being useful to a model.
const MIN_FITTED_EDGE: u32 = 256;
/// Visually transparent for a photograph, and a fraction of the lossless size.
const JPEG_QUALITY: u8 = 90;

pub(crate) fn validate_edit(value: &Value) -> Result<(), String> {
    ImageEdit::parse(value).map(|_| ())
}

/// The encodings a rendered picture can be written in.
///
/// Every one of them earns its place: PNG keeps every pixel, JPEG is the small lossy one, WebP
/// keeps every pixel in less space than PNG, and TIFF is the one print and archival tools accept.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Encoding {
    Png,
    Jpeg,
    WebP,
    Tiff,
}

impl Encoding {
    pub(crate) const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Tiff => "tiff",
        }
    }
}

/// The caller owns authorization; the engine enforces image resource limits.
pub(crate) fn render_source_png(source: &Path, edit: &Value) -> Result<EditedImageOutput, String> {
    render(source, edit, super::MAX_PNG_BYTES)
}

/// One render, written in the encoding the destination asked for.
pub(crate) fn render_source_as(
    source: &Path,
    edit: &Value,
    encoding: Encoding,
) -> Result<Vec<u8>, String> {
    let rendered = rendered(source, edit)?;
    match encoding {
        Encoding::Png => encoded(&rendered, ImageFormat::Png),
        Encoding::Jpeg => jpeg(&rendered),
        Encoding::WebP => encoded(&rendered, ImageFormat::WebP),
        Encoding::Tiff => encoded(&rendered, ImageFormat::Tiff),
    }
}

/// The same render, downscaled until it fits a budget a provider declared.
pub(crate) fn render_source_png_within(
    source: &Path,
    edit: &Value,
    max_bytes: usize,
) -> Result<EditedImageOutput, String> {
    render(source, edit, max_bytes.min(super::MAX_PNG_BYTES))
}

fn render(source: &Path, edit: &Value, max_bytes: usize) -> Result<EditedImageOutput, String> {
    fitted(rendered(source, edit)?, max_bytes)
}

/// Transparency has no JPEG representation, so it is composited onto white rather than turned
/// into noise, and the quality is chosen here rather than left to a default.
fn jpeg(rendered: &image::RgbaImage) -> Result<Vec<u8>, String> {
    let mut flattened = image::RgbImage::new(rendered.width(), rendered.height());
    for (target, source) in flattened.pixels_mut().zip(rendered.pixels()) {
        let alpha = f32::from(source.0[3]) / 255.0;
        *target = image::Rgb(std::array::from_fn(|channel| {
            (f32::from(source.0[channel]) * alpha + 255.0 * (1.0 - alpha)).round() as u8
        }));
    }
    let mut output = Cursor::new(Vec::new());
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, JPEG_QUALITY)
        .write_image(
            flattened.as_raw(),
            flattened.width(),
            flattened.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|error| error.to_string())?;
    Ok(output.into_inner())
}

fn rendered(source: &Path, edit: &Value) -> Result<image::RgbaImage, String> {
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
    pixels::render(&source.to_rgba8(), &edit)
}

/// Encode, and if the result is over budget, downscale and try again. Both
/// sides shrink by the same factor: a reference image that changed shape would
/// be a different picture.
fn fitted(image: image::RgbaImage, max_bytes: usize) -> Result<EditedImageOutput, String> {
    let mut image = image;
    loop {
        let (width, height) = image.dimensions();
        let png = encoded(&image, ImageFormat::Png)?;
        if png.len() <= max_bytes {
            return Ok(EditedImageOutput { png, width, height });
        }
        if width.min(height) <= MIN_FITTED_EDGE {
            return Err(format!(
                "Image does not fit the {max_bytes} byte budget even at its smallest useful size"
            ));
        }
        image = image::imageops::resize(
            &image,
            width * 7 / 10,
            height * 7 / 10,
            image::imageops::FilterType::CatmullRom,
        );
    }
}

fn encoded(image: &image::RgbaImage, format: ImageFormat) -> Result<Vec<u8>, String> {
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, format)
        .map_err(|error| error.to_string())?;
    Ok(output.into_inner())
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

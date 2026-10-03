mod pixels;

#[cfg(test)]
mod tests;

use std::io::Cursor;

use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, Limits};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::BackgroundRemovalOutput;
use crate::{
    ai::contracts::image_wire::ImageSegmentationExecutionIdentity,
    local_resources::preparation::PreparedModels,
};

const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_IMAGE_PIXELS: u64 = 100_000_000;
const MAX_IMAGE_DIMENSION: u32 = 10_000;
const MAX_OUTPUT_BYTES: usize = 64 * 1024 * 1024;

pub(crate) async fn remove_background<R: Runtime>(
    app: &AppHandle<R>,
    source: Vec<u8>,
    identity: &ImageSegmentationExecutionIdentity,
    resources: &PreparedModels,
    cancellation: &CancellationToken,
) -> Result<BackgroundRemovalOutput, String> {
    let (image, input) = crate::blocking::run(move || prepare_input(&source)).await?;
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    let mask =
        crate::ai::local_runtime::segment_image(app, input, identity, resources, cancellation)
            .await?;
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    let output = crate::blocking::run(move || compose_output(image, mask)).await?;
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    Ok(output)
}

fn prepare_input(source: &[u8]) -> Result<(DynamicImage, Vec<f32>), String> {
    if source.is_empty() || source.len() as u64 > MAX_SOURCE_BYTES {
        return Err("Background-removal input must be a non-empty image up to 64 MiB".into());
    }
    let mut reader = ImageReader::new(Cursor::new(source))
        .with_guessed_format()
        .map_err(|error| error.to_string())?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    reader.limits(limits);
    let image = reader.decode().map_err(|error| error.to_string())?;
    validate_dimensions(image.dimensions())?;
    let input = pixels::model_input(&image);
    Ok((image, input))
}

fn compose_output(image: DynamicImage, mask: Vec<f32>) -> Result<BackgroundRemovalOutput, String> {
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

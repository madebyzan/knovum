//! A subject's coverage from a background-removal model, and the background removal built on it.
//! Layer splitting reads the same coverage as its subject's soft edge, so both run one path.

mod field;

#[cfg(test)]
mod tests;

use std::{io::Cursor, sync::Arc};

use image::{GenericImageView, ImageFormat, ImageReader, Limits, RgbaImage};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::{cutout::cut_out, BackgroundRemovalOutput, MAX_PNG_BYTES};
use crate::{
    ai::contracts::image_wire::ImageSegmentationExecutionIdentity,
    local_resources::{models::catalog::SegmentationModel, preparation::PreparedModels},
};

const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_IMAGE_PIXELS: u64 = 100_000_000;
const MAX_IMAGE_DIMENSION: u32 = 10_000;

/// How much of every source pixel belongs to the subject, in `[0, 1]`, row-major at the source's
/// resolution.
pub(crate) async fn subject_coverage<R: Runtime>(
    app: &AppHandle<R>,
    source: Arc<RgbaImage>,
    identity: &ImageSegmentationExecutionIdentity,
    resources: &PreparedModels,
    cancellation: &CancellationToken,
) -> Result<Vec<f32>, String> {
    let spec = SegmentationModel::from_spec_id(identity.model_spec_id)?.spec();
    let dimensions = source.dimensions();
    let input = crate::foundation::blocking::run(move || {
        Ok::<_, String>(field::model_input(spec, &source))
    })
    .await?;
    let output =
        crate::ai::local_runtime::segment_image(app, input, identity, resources, cancellation)
            .await?;
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    crate::foundation::blocking::run(move || field::coverage(spec, &output, dimensions)).await
}

/// The subject of a rendered PNG cut out of its backdrop, as a PNG of the same size.
pub(crate) async fn remove_background<R: Runtime>(
    app: &AppHandle<R>,
    source: Vec<u8>,
    identity: &ImageSegmentationExecutionIdentity,
    resources: &PreparedModels,
    cancellation: &CancellationToken,
) -> Result<BackgroundRemovalOutput, String> {
    let image = Arc::new(crate::foundation::blocking::run(move || decode(&source)).await?);
    let coverage =
        subject_coverage(app, Arc::clone(&image), identity, resources, cancellation).await?;
    crate::foundation::blocking::run(move || encode(&cut_out(&image, &coverage))).await
}

fn decode(source: &[u8]) -> Result<RgbaImage, String> {
    if source.is_empty() || source.len() as u64 > MAX_SOURCE_BYTES {
        return Err("Background-removal input must be a non-empty PNG up to 64 MiB".into());
    }
    // The source is always the engine-rendered PNG of what the user sees, so orientation is
    // already applied and no other format is accepted at this boundary.
    let mut reader = ImageReader::with_format(Cursor::new(source), ImageFormat::Png);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    reader.limits(limits);
    let image = reader.decode().map_err(|error| error.to_string())?;
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_IMAGE_PIXELS {
        return Err("Background-removal image dimensions are invalid".into());
    }
    Ok(image.into_rgba8())
}

fn encode(foreground: &RgbaImage) -> Result<BackgroundRemovalOutput, String> {
    let mut png = Cursor::new(Vec::new());
    foreground
        .write_to(&mut png, ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    let png = png.into_inner();
    if png.len() > MAX_PNG_BYTES {
        return Err("Background-removal PNG exceeds 64 MiB".into());
    }
    Ok(BackgroundRemovalOutput {
        png,
        width: foreground.width(),
        height: foreground.height(),
    })
}

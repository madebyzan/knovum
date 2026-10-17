use tokio_util::sync::CancellationToken;

use super::{contracts::ImageOperation, source::SourceImage};
use crate::ai::{capabilities::image, contracts::ImageReference, models::ModelExecutionTarget};
use crate::foundation::app_error::AppError;

/// Every remote image operation is one request: these pixels, this instruction.
/// The configured model decides the transport, so there is nothing to route here.
pub(super) async fn execute(
    target: &ModelExecutionTarget,
    operation: &ImageOperation,
    source: &SourceImage,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, AppError> {
    let (max_images, max_bytes) = image::reference_budget(target).ok_or_else(|| {
        AppError::new(
            "capability_not_supported",
            "Configured image model does not accept an image to work from",
        )
    })?;
    if max_images == 0 || source.bytes.len() > max_bytes {
        return Err(AppError::invalid(format!(
            "Image operation source exceeds the {max_bytes} byte limit of the configured model"
        )));
    }
    let generated = image::generate(
        image::ImageGenerationRequest {
            target: target.clone(),
            prompt: instruction(operation)?,
            aspect_ratio: "1:1".into(),
            references: vec![ImageReference {
                png: source.bytes.clone(),
                width: source.width,
                height: source.height,
            }],
        },
        cancellation,
    )
    .await?;
    Ok(generated.bytes)
}

fn instruction(operation: &ImageOperation) -> Result<String, AppError> {
    match operation {
        ImageOperation::MultiAngle { .. } => operation
            .view_prompt()
            .ok_or_else(|| "Image view prompt is unavailable".into()),
        ImageOperation::RemoveObjectPrompt { prompt } => Ok(format!(
            "Remove this from the provided image: {prompt}. Fill the area with a natural continuation of the surrounding background. Preserve the rest of the image, its composition, lighting, and style."
        )),
        _ => Err(AppError::new(
            "capability_not_supported",
            "This image operation does not run on a remote model",
        )),
    }
}

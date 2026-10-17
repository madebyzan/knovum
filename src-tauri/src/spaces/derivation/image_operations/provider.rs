use tokio_util::sync::CancellationToken;

use super::{contracts::ImageOperation, source::SourceImage};
use crate::ai::{capabilities::image, contracts::ImageReference, models::ModelExecutionTarget};
use crate::foundation::app_error::AppError;

/// Every remote image operation is one request: these pixels, this instruction.
/// The shared image capability owns the budget and the transport, so there is
/// nothing to route or re-check here.
pub(super) async fn execute(
    target: &ModelExecutionTarget,
    operation: &ImageOperation,
    source: &SourceImage,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, AppError> {
    let generated = image::generate(
        image::ImageGenerationRequest {
            target: target.clone(),
            prompt: instruction(operation)?,
            // Changing a picture keeps the picture's frame.
            aspect_ratio: None,
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
        ImageOperation::Upscale { .. }
        | ImageOperation::RemoveObject { .. }
        | ImageOperation::RemoveBackground
        | ImageOperation::ExtractSelection { .. }
        | ImageOperation::RemoveSelectedObject { .. }
        | ImageOperation::GenerateDepth => Err(AppError::new(
            "capability_not_supported",
            "This image operation runs on the local engine, not on a model",
        )),
    }
}

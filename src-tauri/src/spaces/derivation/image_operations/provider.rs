use tokio_util::sync::CancellationToken;

use super::{contracts::ImageOperation, routing::ImageOperationBackend, source::SourceImage};
use crate::ai::{
    models::ModelExecutionTarget,
    providers::{dashscope::execute_image_view, gemini, image_edit, kling},
    transport::SecurityContext,
};
use crate::foundation::app_error::AppError;

pub(super) async fn execute(
    target: &ModelExecutionTarget,
    backend: ImageOperationBackend,
    operation: &ImageOperation,
    source: &SourceImage,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, AppError> {
    if source.bytes.len() > 10 * 1024 * 1024 {
        return Err("Image operation source exceeds the 10 MB provider limit".into());
    }
    let ctx = SecurityContext::from_target(target).await?;
    let bytes = match (backend, operation) {
        (ImageOperationBackend::DashScope, ImageOperation::MultiAngle { .. }) => {
            let prompt = operation
                .view_prompt()
                .ok_or("Image view prompt is unavailable")?;
            let data_uri = source.data_uri();
            execute_image_view(&ctx, &data_uri, &prompt, cancellation).await?
        }
        (ImageOperationBackend::Gemini, ImageOperation::RemoveObjectPrompt { prompt }) => {
            gemini::edit_image(
                &ctx,
                &source.bytes,
                &removal_instruction(prompt),
                cancellation,
            )
            .await?
        }
        (ImageOperationBackend::Kling, ImageOperation::RemoveObjectPrompt { prompt }) => {
            kling::edit_image(
                &ctx,
                &source.bytes,
                source.width,
                source.height,
                &removal_instruction(prompt),
                cancellation,
            )
            .await?
        }
        (ImageOperationBackend::ImageEdit(api), ImageOperation::RemoveObjectPrompt { prompt }) => {
            image_edit::edit_image(
                api,
                &ctx,
                &source.bytes,
                &removal_instruction(prompt),
                cancellation,
            )
            .await?
        }
        _ => {
            return Err(AppError::new(
                "capability_not_supported",
                "Configured image model does not support this image operation",
            ))
        }
    };
    Ok(bytes)
}

fn removal_instruction(prompt: &str) -> String {
    format!(
        "Remove this from the provided image: {prompt}. Fill the area with a natural continuation of the surrounding background. Preserve the rest of the image, its composition, lighting, and style."
    )
}

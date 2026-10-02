use tokio_util::sync::CancellationToken;

use super::{contracts::ImageOperation, routing::ImageOperationBackend, source::SourceImage};
use crate::ai::{
    models::ModelExecutionTarget,
    providers::{
        dashscope::{execute_image_operation, SourceImageOperation},
        gemini, image_edit, kling, openai,
    },
    transport::SecurityContext,
};
use crate::app_error::AppError;

pub(super) async fn execute(
    target: &ModelExecutionTarget,
    backend: ImageOperationBackend,
    operation: &ImageOperation,
    source: &SourceImage,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, AppError> {
    let ctx = SecurityContext::from_target(target).await?;
    let bytes = match (backend, operation) {
        (ImageOperationBackend::DashScope, ImageOperation::Upscale { target_long_edge }) => {
            super::upscale::execute(&ctx, source, *target_long_edge, cancellation).await?
        }
        (ImageOperationBackend::DashScope, ImageOperation::MultiAngle { .. }) => {
            let prompt = operation
                .view_prompt()
                .ok_or("Image view prompt is unavailable")?;
            let data_uri = source.data_uri();
            execute_image_operation(
                &ctx,
                SourceImageOperation::View {
                    data_uri: &data_uri,
                    prompt: &prompt,
                },
                cancellation,
            )
            .await?
        }
        (
            ImageOperationBackend::OpenAi | ImageOperationBackend::DashScope,
            ImageOperation::RemoveObject { strokes },
        ) => {
            let (width, height, strokes) = (source.width, source.height, strokes.clone());
            if cancellation.is_cancelled() {
                return Err(AppError::cancelled("Image operation was cancelled"));
            }
            let mask_cancellation = cancellation.clone();
            if backend == ImageOperationBackend::OpenAi {
                let mask = crate::blocking::run(move || {
                    super::mask::encode_openai(width, height, &strokes, &mask_cancellation)
                })
                .await?;
                if cancellation.is_cancelled() {
                    return Err(AppError::cancelled("Image operation was cancelled"));
                }
                openai::edit_mask(&ctx, &source.bytes, mask, cancellation).await?
            } else {
                let mask = crate::blocking::run(move || {
                    super::mask::encode(width, height, &strokes, &mask_cancellation)
                })
                .await?;
                if cancellation.is_cancelled() {
                    return Err(AppError::cancelled("Image operation was cancelled"));
                }
                let data_uri = source.data_uri();
                execute_image_operation(
                    &ctx,
                    SourceImageOperation::Inpaint {
                        data_uri: &data_uri,
                        mask_uri: &mask,
                    },
                    cancellation,
                )
                .await?
            }
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

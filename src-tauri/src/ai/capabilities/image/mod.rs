use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{GeneratedImage, ImageGenerationInput},
    models::ModelExecutionTarget,
    providers::resolve_provider,
    transport::SecurityContext,
};
use crate::foundation::app_error::{AppError, AppResult};

#[derive(Debug)]
pub struct ImageGenerationRequest {
    pub target: ModelExecutionTarget,
    pub prompt: String,
    pub aspect_ratio: String,
}

pub async fn generate(
    _app: &AppHandle,
    request: ImageGenerationRequest,
    cancellation: &CancellationToken,
) -> AppResult<GeneratedImage> {
    validate(&request)?;
    let kind = request.target.model_provider_kind()?;
    let provider = resolve_provider(kind);
    let image_generation = provider.as_image_generation().ok_or_else(|| {
        AppError::capability_not_supported(request.target.provider(), "image generation")
    })?;

    let ctx = SecurityContext::from_target(&request.target).await?;
    let req = ImageGenerationInput {
        prompt: request.prompt,
        aspect_ratio: request.aspect_ratio,
    };

    tokio::select! {
        _ = cancellation.cancelled() => Err(AppError::cancelled("Image generation was cancelled")),
        generated_image = image_generation.generate(&ctx, &req) => generated_image,
    }
}

fn validate(request: &ImageGenerationRequest) -> AppResult<()> {
    if request.prompt.trim().is_empty() {
        return Err(AppError::new("invalid_request", "Image prompt is required"));
    }
    if request.aspect_ratio.trim().is_empty() {
        return Err(AppError::new(
            "invalid_request",
            "Image aspect ratio is required",
        ));
    }
    Ok(())
}

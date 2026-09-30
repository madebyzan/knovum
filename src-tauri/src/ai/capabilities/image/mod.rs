use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::ImageGenerationInput,
    error::{AiError, AiErrorCode, AiResult},
    models::ModelExecutionTarget,
    providers::resolve_provider,
    transport::SecurityContext,
};

pub use crate::ai::contracts::GeneratedImage;

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
) -> AiResult<GeneratedImage> {
    validate(&request)?;
    let kind = request.target.model_provider_kind()?;
    let provider = resolve_provider(kind);
    let image_generation = provider.as_image_generation().ok_or_else(|| {
        AiError::capability_not_supported(request.target.provider(), "image generation")
    })?;

    let ctx = SecurityContext::from_target(&request.target).await?;
    let req = ImageGenerationInput {
        prompt: request.prompt,
        aspect_ratio: request.aspect_ratio,
    };

    tokio::select! {
        _ = cancellation.cancelled() => Err(AiError::cancelled("Image generation was cancelled")),
        generated_image = image_generation.generate(&ctx, &req) => generated_image,
    }
}

fn validate(request: &ImageGenerationRequest) -> AiResult<()> {
    if request.prompt.trim().is_empty() {
        return Err(AiError::new(
            AiErrorCode::InvalidRequest,
            "Image prompt is required",
        ));
    }
    if request.aspect_ratio.trim().is_empty() {
        return Err(AiError::new(
            AiErrorCode::InvalidRequest,
            "Image aspect ratio is required",
        ));
    }
    Ok(())
}

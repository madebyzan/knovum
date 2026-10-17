use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{GeneratedImage, ImageGenerationInput, ImageReference},
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
    /// Pixels the model must see, already rendered by the caller.
    pub references: Vec<ImageReference>,
}

/// How many references the configured model accepts, and how large each may be.
/// Callers render within this budget instead of discovering a provider limit
/// through a failed request.
pub fn reference_budget(target: &ModelExecutionTarget) -> Option<(usize, usize)> {
    let provider = resolve_provider(target.model_provider_kind().ok()?);
    let support = provider
        .as_image_generation()?
        .reference_support(target.provider(), target.model())?;
    Some((support.max_images.into(), support.max_bytes_per_image))
}

pub async fn generate(
    request: ImageGenerationRequest,
    cancellation: &CancellationToken,
) -> AppResult<GeneratedImage> {
    validate(&request)?;
    let kind = request.target.model_provider_kind()?;
    let provider = resolve_provider(kind);
    let image_generation = provider.as_image_generation().ok_or_else(|| {
        AppError::capability_not_supported(request.target.provider(), "image generation")
    })?;
    if !request.references.is_empty() {
        let support = image_generation
            .reference_support(request.target.provider(), request.target.model())
            .ok_or_else(|| {
                AppError::capability_not_supported(
                    request.target.provider(),
                    "image generation from a reference image",
                )
            })?;
        if request.references.len() > usize::from(support.max_images) {
            return Err(AppError::invalid(format!(
                "{} accepts at most {} reference images",
                request.target.model(),
                support.max_images
            )));
        }
        if let Some(oversized) = request
            .references
            .iter()
            .find(|reference| reference.png.len() > support.max_bytes_per_image)
        {
            return Err(AppError::invalid(format!(
                "A {} byte reference image exceeds the {} byte limit of {}",
                oversized.png.len(),
                support.max_bytes_per_image,
                request.target.model()
            )));
        }
    }

    let ctx = SecurityContext::from_target(&request.target).await?;
    let req = ImageGenerationInput {
        prompt: request.prompt,
        aspect_ratio: request.aspect_ratio,
        references: request.references,
    };
    // The token lets an adapter abort uploads and polling; the select guarantees
    // the Run stops waiting even for an adapter that cannot interrupt a request.
    tokio::select! {
        _ = cancellation.cancelled() => Err(AppError::cancelled("Image generation was cancelled")),
        generated_image = image_generation.generate(&ctx, &req, cancellation) => generated_image,
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

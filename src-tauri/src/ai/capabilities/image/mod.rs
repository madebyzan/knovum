use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{GeneratedImage, ImageGenerationInput, ImageReference, ImageReferenceSupport},
    models::ModelExecutionTarget,
    providers::resolve_provider,
    transport::SecurityContext,
};
use crate::foundation::app_error::{AppError, AppResult};

#[derive(Debug)]
pub struct ImageGenerationRequest {
    pub target: ModelExecutionTarget,
    pub prompt: String,
    /// The frame the user asked for. A request built from pixels asks for none:
    /// the attached image decides the frame.
    pub aspect_ratio: Option<String>,
    /// Pixels the model must see, already rendered by the caller.
    pub references: Vec<ImageReference>,
}

/// What the configured model accepts alongside the prompt, if anything. Callers
/// render within this budget instead of discovering a provider limit through a
/// failed request.
pub fn reference_support(target: &ModelExecutionTarget) -> Option<ImageReferenceSupport> {
    resolve_provider(target.model_provider_kind().ok()?)
        .as_image_generation()?
        .reference_support(target.provider(), target.model())
}

pub async fn generate(
    request: ImageGenerationRequest,
    cancellation: &CancellationToken,
) -> AppResult<GeneratedImage> {
    let kind = request.target.model_provider_kind()?;
    let provider = resolve_provider(kind);
    let image_generation = provider.as_image_generation().ok_or_else(|| {
        AppError::capability_not_supported(request.target.provider(), "image generation")
    })?;
    validate(
        &request,
        image_generation.reference_support(request.target.provider(), request.target.model()),
    )?;

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

/// The request must be one the configured model can answer: words need a frame,
/// pixels need a model that reads them, and the budget is the model's own.
fn validate(
    request: &ImageGenerationRequest,
    support: Option<ImageReferenceSupport>,
) -> AppResult<()> {
    if request.prompt.trim().is_empty() {
        return Err(AppError::new("invalid_request", "Image prompt is required"));
    }
    let model = request.target.model();
    if request.references.is_empty() {
        return match &request.aspect_ratio {
            Some(ratio) if !ratio.trim().is_empty() => Ok(()),
            _ => Err(AppError::new(
                "invalid_request",
                "A text-to-image request needs an aspect ratio",
            )),
        };
    }
    let support = support.ok_or_else(|| {
        AppError::capability_not_supported(
            request.target.provider(),
            "image generation from a reference image",
        )
    })?;
    if request.references.len() > usize::from(support.max_images) {
        return Err(AppError::invalid(format!(
            "{model} accepts at most {} reference images",
            support.max_images
        )));
    }
    match request
        .references
        .iter()
        .find(|reference| reference.png.len() > support.max_bytes_per_image)
    {
        Some(oversized) => Err(AppError::invalid(format!(
            "A {} byte reference image exceeds the {} byte limit of {model}",
            oversized.png.len(),
            support.max_bytes_per_image,
        ))),
        None => Ok(()),
    }
}

use futures::future::BoxFuture;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use super::endpoint;
use crate::ai::providers::image_references::{self, ReferenceApi, MAX_INLINE_REFERENCE_BYTES};
use crate::ai::{
    contracts::{
        GeneratedImage, ImageGenerationCapability, ImageGenerationInput, ImageReferenceSupport,
    },
    transport::{http_client, response, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};

/// The ratios OpenRouter's `/images` endpoint accepts; anything else is left
/// out so the provider applies its own default.
const ASPECT_RATIOS: [&str; 9] = [
    "1:1", "16:9", "9:16", "4:3", "3:4", "3:2", "2:3", "21:9", "9:21",
];

pub struct OpenRouterImageAdapter;

#[derive(Deserialize)]
struct ImageResponse {
    data: Vec<GeneratedImageSource>,
}

#[derive(Deserialize)]
struct GeneratedImageSource {
    b64_json: String,
}

impl ImageGenerationCapability for OpenRouterImageAdapter {
    fn reference_support(&self, provider: &str, model: &str) -> Option<ImageReferenceSupport> {
        ReferenceApi::for_model(provider, model).map(|_| ImageReferenceSupport {
            max_images: 1,
            max_bytes_per_image: MAX_INLINE_REFERENCE_BYTES,
        })
    }

    fn generate<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ImageGenerationInput,
        cancellation: &'a CancellationToken,
    ) -> BoxFuture<'a, AppResult<GeneratedImage>> {
        Box::pin(async move {
            if let Some(reference) = req.references.first() {
                let api = ReferenceApi::for_model(&ctx.provider, &ctx.model).ok_or_else(|| {
                    AppError::capability_not_supported(
                        &ctx.provider,
                        "image generation from a reference image",
                    )
                })?;
                let bytes = image_references::generate_from_reference(
                    api,
                    ctx,
                    &reference.png,
                    &req.prompt,
                    cancellation,
                )
                .await
                .map_err(|error| AppError::new("provider_request_failed", error))?;
                return Ok(GeneratedImage {
                    bytes,
                    provider: ctx.provider.clone(),
                    model: ctx.model.clone(),
                    revised_prompt: None,
                });
            }

            let mut payload = serde_json::json!({
                "model": ctx.model,
                "prompt": req.prompt,
            });
            let ratio = req.require_aspect_ratio()?;
            if ASPECT_RATIOS.contains(&ratio) {
                payload["aspect_ratio"] = serde_json::json!(ratio);
            }

            let send = http_client()
                .post(format!("{}/images", endpoint(&ctx.endpoint)))
                .bearer_auth(ctx.require_api_key()?)
                .json(&payload)
                .send();
            let generation_response = tokio::select! {
                _ = cancellation.cancelled() => {
                    return Err(AppError::cancelled("Image generation was cancelled"))
                }
                response = send => response
                    .map_err(|error| AppError::new("provider_request_failed", error.to_string()))?,
            };
            if !generation_response.status().is_success() {
                return Err(AppError::new(
                    "provider_request_failed",
                    response::read_text(generation_response).await?,
                ));
            }
            let body: ImageResponse =
                response::read_json(generation_response, response::MAX_IMAGE_JSON_BYTES).await?;
            let item = body.data.into_iter().next().ok_or_else(|| {
                AppError::new("empty_response", "AI image provider returned no image")
            })?;
            let bytes = response::decode_image(&item.b64_json)?;
            Ok(GeneratedImage {
                bytes,
                provider: ctx.provider.clone(),
                model: ctx.model.clone(),
                revised_prompt: None,
            })
        })
    }
}

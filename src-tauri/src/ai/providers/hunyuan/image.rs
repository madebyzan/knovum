use base64::{engine::general_purpose::STANDARD, Engine};
use futures::future::BoxFuture;
use serde::Deserialize;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::endpoint;
use crate::ai::{
    contracts::{
        GeneratedImage, ImageGenerationCapability, ImageGenerationInput, ImageReferenceSupport,
    },
    transport::{http_client, response, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};

/// Hy-Image-3.0 on TokenHub: synchronous text-to-image, optionally seeded with
/// up to three reference images.
pub struct HunyuanImageAdapter;

#[derive(Deserialize)]
struct ImageResponse {
    #[serde(default)]
    data: Vec<ImageItem>,
}

#[derive(Deserialize)]
struct ImageItem {
    url: Option<String>,
    #[serde(default)]
    revised_prompt: Option<String>,
}

impl ImageGenerationCapability for HunyuanImageAdapter {
    fn generate<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ImageGenerationInput,
        _cancellation: &'a CancellationToken,
    ) -> BoxFuture<'a, AppResult<GeneratedImage>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let mut body = json!({
                "model": ctx.model,
                "prompt": req.prompt,
                "size": size_for_ratio(req.require_aspect_ratio()?),
            });
            if !req.references.is_empty() {
                body["images"] = json!(req
                    .references
                    .iter()
                    .map(|image| format!("data:image/png;base64,{}", STANDARD.encode(&image.png)))
                    .collect::<Vec<_>>());
            }

            let endpoint = endpoint(&ctx.endpoint);
            let generated = http_client()
                .post(format!("{endpoint}/wand/hunyuan-image/v3-generation"))
                .bearer_auth(secret)
                .json(&body)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let parsed: ImageResponse =
                response::read_json(generated, response::MAX_CONTROL_JSON_BYTES).await?;
            let item = parsed.data.into_iter().next().ok_or_else(|| {
                AppError::new("invalid_response", "TokenHub returned no image URL")
            })?;
            let url = item.url.ok_or_else(|| {
                AppError::new("invalid_response", "TokenHub returned no image URL")
            })?;

            let bytes = response::download_image(&url).await?;
            Ok(GeneratedImage {
                bytes,
                provider: ctx.provider.clone(),
                model: ctx.model.clone(),
                revised_prompt: item.revised_prompt,
            })
        })
    }

    fn reference_support(&self, _provider: &str, _model: &str) -> Option<ImageReferenceSupport> {
        Some(ImageReferenceSupport {
            max_images: 3,
            max_bytes_per_image: 10 * 1024 * 1024,
        })
    }
}

/// The API takes a pixel size instead of a ratio; sides stay within
/// [512, 2048] and the area within 1024x1024 as the API requires.
fn size_for_ratio(ratio: &str) -> &'static str {
    match ratio {
        "16:9" => "1280x720",
        "9:16" => "720x1280",
        "4:3" => "1024x768",
        "3:4" => "768x1024",
        _ => "1024x1024",
    }
}

fn provider_err(error: impl std::fmt::Display) -> AppError {
    AppError::new("provider_request_failed", error.to_string())
}

#[cfg(test)]
#[path = "tests/image.rs"]
mod tests;

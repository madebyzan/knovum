use futures::future::BoxFuture;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use super::super::image_references::{self, ReferenceApi, MAX_INLINE_REFERENCE_BYTES};
use crate::ai::{
    contracts::{
        GeneratedImage, ImageGenerationCapability, ImageGenerationInput, ImageReferenceSupport,
    },
    transport::{endpoint_with_suffix, http_client, response, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};

pub struct OpenAiImageAdapter;

#[derive(Deserialize)]
struct ImageResponse {
    #[serde(default)]
    data: Vec<GeneratedImageSource>,
    #[serde(default)]
    images: Vec<GeneratedImageSource>,
}

#[derive(Deserialize)]
struct GeneratedImageSource {
    url: Option<String>,
    b64_json: Option<String>,
    revised_prompt: Option<String>,
}

impl ImageGenerationCapability for OpenAiImageAdapter {
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
            let api_key = ctx.require_api_key()?;
            let aspect_ratio = req.require_aspect_ratio()?;
            let payload = if ctx.provider == "grok" {
                // xAI images take an aspect ratio and resolution, never pixel
                // dimensions, and can answer inline with base64.
                serde_json::json!({
                    "prompt": req.prompt,
                    "model": ctx.model,
                    "n": 1,
                    "aspect_ratio": aspect_ratio,
                    "response_format": "b64_json",
                })
            } else {
                let dimensions = image_size(&ctx.provider, &ctx.model, aspect_ratio);
                let mut payload = serde_json::json!({
                    "prompt": req.prompt,
                    "model": ctx.model,
                    "n": 1,
                    "size": dimensions,
                });
                if ctx.provider == "siliconflow" {
                    payload["image_size"] = serde_json::json!(dimensions);
                } else if ctx.provider == "together" {
                    let (width, height) = parse_dimensions(dimensions);
                    payload["width"] = serde_json::json!(width);
                    payload["height"] = serde_json::json!(height);
                } else if matches!(ctx.provider.as_str(), "openai" | "openai-compatible")
                    && !ctx.model.starts_with("gpt-image-")
                {
                    // gpt-image models reject `response_format` and always answer
                    // with base64; DALL-E needs the parameter to avoid a URL.
                    payload["response_format"] = serde_json::json!("b64_json");
                }
                payload
            };

            let endpoint = image_endpoint(&ctx.provider, &ctx.endpoint);
            let url = format!("{endpoint}/images/generations");
            let response = http_client()
                .post(url)
                .bearer_auth(api_key)
                .json(&payload)
                .send()
                .await
                .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?;

            let status = response.status();
            if !status.is_success() {
                let text = response::read_text(response).await?;
                return Err(AppError::new("provider_request_failed", text));
            }

            let body: ImageResponse =
                response::read_json(response, response::MAX_IMAGE_JSON_BYTES).await?;

            let item = body
                .data
                .into_iter()
                .chain(body.images)
                .next()
                .ok_or_else(|| {
                    AppError::new("empty_response", "AI image provider returned no image")
                })?;

            let bytes = if let Some(b64) = item.b64_json {
                response::decode_image(&b64)?
            } else if let Some(image_url) = item.url {
                response::download_image(&image_url).await?
            } else {
                return Err(AppError::new(
                    "invalid_response",
                    "Image response contained neither b64_json nor url",
                ));
            };

            Ok(GeneratedImage {
                bytes,
                provider: ctx.provider.clone(),
                model: ctx.model.clone(),
                revised_prompt: item.revised_prompt,
            })
        })
    }
}

fn parse_dimensions(dimensions: &str) -> (u32, u32) {
    match dimensions {
        "1344x768" => (1344, 768),
        "768x1344" => (768, 1344),
        "1792x1024" => (1792, 1024),
        "1024x1792" => (1024, 1792),
        _ => (1024, 1024),
    }
}

fn image_endpoint(provider: &str, raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    match provider {
        "zhipu" => endpoint_with_suffix(trimmed, "/api/paas/v4"),
        "doubao" => endpoint_with_suffix(trimmed, "/api/v3"),
        _ => {
            if has_version_segment(trimmed) {
                trimmed.to_string()
            } else {
                format!("{trimmed}/v1")
            }
        }
    }
}

/// Bases such as Baidu's `/v2` already carry their API version; appending `/v1`
/// there would address a nonexistent endpoint.
fn has_version_segment(base: &str) -> bool {
    match base.rsplit_once('/') {
        Some((_, last)) => {
            last.len() > 1 && last.starts_with('v') && last[1..].chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

/// Each OpenAI image family accepts its own fixed size menu; sending a size
/// from another family's menu is rejected outright.
fn image_size(provider: &str, model: &str, aspect_ratio: &str) -> &'static str {
    match (provider, model) {
        ("openai" | "openai-compatible", model) if model.starts_with("gpt-image-") => {
            match aspect_ratio {
                "3:2" | "16:9" | "4:3" => "1536x1024",
                "2:3" | "9:16" | "3:4" => "1024x1536",
                _ => "1024x1024",
            }
        }
        ("openai" | "openai-compatible", "dall-e-2") => "1024x1024",
        ("openai" | "openai-compatible", _) => match aspect_ratio {
            "3:2" | "16:9" | "4:3" => "1792x1024",
            "2:3" | "9:16" | "3:4" => "1024x1792",
            _ => "1024x1024",
        },
        _ => match aspect_ratio {
            "3:2" | "16:9" => "1344x768",
            "2:3" | "9:16" => "768x1344",
            _ => "1024x1024",
        },
    }
}

use futures::future::BoxFuture;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{GeneratedImage, ImageGenerationCapability, ImageGenerationInput},
    transport::{http_client, response, SecurityContext},
};
use crate::app_error::{AppError, AppResult};

pub struct OpenAiImageAdapter;

pub async fn edit_mask(
    ctx: &SecurityContext,
    image: &[u8],
    mask: Vec<u8>,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, String> {
    let part = |bytes: Vec<u8>, name: &str| {
        Part::bytes(bytes)
            .file_name(name.to_owned())
            .mime_str("image/png")
            .map_err(|error| error.to_string())
    };
    let form = Form::new()
        .text("model", ctx.model.clone())
        .text("prompt", "Recreate the original image with the masked area replaced by a natural, unobstructed continuation of the surrounding background. Preserve everything outside the mask.")
        .part("image[]", part(image.to_vec(), "source.png")?)
        .part("mask", part(mask, "mask.png")?);
    let request = http_client()
        .post(format!(
            "{}/images/edits",
            image_endpoint("openai", &ctx.endpoint)
        ))
        .bearer_auth(ctx.require_api_key().map_err(|error| error.to_string())?)
        .multipart(form)
        .send();
    let result = tokio::select! {
        _ = cancellation.cancelled() => return Err("Image operation was cancelled".into()),
        response = request => response.map_err(|error| error.to_string())?,
    };
    if !result.status().is_success() {
        return Err(response::read_text(result)
            .await
            .map_err(|error| error.to_string())?);
    }
    let body: ImageResponse = response::read_json(result, response::MAX_IMAGE_JSON_BYTES)
        .await
        .map_err(|error| error.to_string())?;
    let encoded = body
        .data
        .into_iter()
        .next()
        .and_then(|image| image.b64_json)
        .ok_or("OpenAI image edit returned no image")?;
    response::decode_image(&encoded).map_err(|error| error.to_string())
}

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
    fn generate<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ImageGenerationInput,
    ) -> BoxFuture<'a, AppResult<GeneratedImage>> {
        Box::pin(async move {
            let api_key = ctx.require_api_key()?;
            let dimensions = image_size(&ctx.provider, &ctx.model, &req.aspect_ratio);
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
            } else if matches!(ctx.provider.as_str(), "openai" | "openai-compatible") {
                payload["response_format"] = serde_json::json!("b64_json");
            }

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
                let download_response = http_client()
                    .get(image_url)
                    .send()
                    .await
                    .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?
                    .error_for_status()
                    .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?;
                response::read_bytes(download_response, response::MAX_IMAGE_BYTES).await?
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
        "zhipu" => {
            if trimmed.ends_with("/api/paas/v4") {
                trimmed.to_string()
            } else {
                format!("{trimmed}/api/paas/v4")
            }
        }
        "doubao" => {
            if trimmed.ends_with("/api/v3") {
                trimmed.to_string()
            } else {
                format!("{trimmed}/api/v3")
            }
        }
        _ => {
            if trimmed.ends_with("/v1") {
                trimmed.to_string()
            } else {
                format!("{trimmed}/v1")
            }
        }
    }
}

fn image_size(provider: &str, model: &str, aspect_ratio: &str) -> &'static str {
    if matches!(provider, "openai" | "openai-compatible") && !model.contains("dall-e-2") {
        match aspect_ratio {
            "3:2" | "16:9" => "1792x1024",
            "2:3" | "9:16" => "1024x1792",
            _ => "1024x1024",
        }
    } else {
        match aspect_ratio {
            "3:2" | "16:9" => "1344x768",
            "2:3" | "9:16" => "768x1344",
            _ => "1024x1024",
        }
    }
}

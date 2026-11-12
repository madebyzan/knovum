use futures::future::BoxFuture;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use super::{endpoint, image_operation::generate_from_reference};
use crate::ai::{
    contracts::{
        GeneratedImage, ImageGenerationCapability, ImageGenerationInput, ImageReferenceSupport,
    },
    transport::{http_client, response, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};

pub struct DashScopeImageAdapter;

/// The qwen-image protocol takes one uploaded image per request.
const MAX_REFERENCE_BYTES: usize = 10 * 1024 * 1024;

#[derive(Deserialize)]
struct ResponsePayload {
    output: OutputPayload,
    code: Option<String>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct OutputPayload {
    choices: Option<Vec<GenerationChoice>>,
    #[serde(rename = "results")]
    artifacts: Option<Vec<ImageArtifact>>,
}

#[derive(Deserialize)]
struct GenerationChoice {
    message: GenerationMessage,
}

#[derive(Deserialize)]
struct GenerationMessage {
    content: Vec<ImageContentPart>,
}

#[derive(Deserialize)]
struct ImageContentPart {
    image: Option<String>,
}

#[derive(Deserialize)]
struct ImageArtifact {
    url: String,
    actual_prompt: Option<String>,
}

impl ImageGenerationCapability for DashScopeImageAdapter {
    fn reference_support(&self, _provider: &str, _model: &str) -> Option<ImageReferenceSupport> {
        Some(ImageReferenceSupport {
            max_images: 1,
            max_bytes_per_image: MAX_REFERENCE_BYTES,
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
                let bytes =
                    generate_from_reference(ctx, reference.png.clone(), &req.prompt, cancellation)
                        .await
                        .map_err(|error| AppError::new("provider_request_failed", error))?;
                return Ok(GeneratedImage {
                    bytes,
                    provider: ctx.provider.clone(),
                    model: ctx.model.clone(),
                    revised_prompt: None,
                });
            }
            let secret = ctx.require_api_key()?;
            let endpoint = endpoint(&ctx.endpoint);
            let body = serde_json::json!({
                "model": ctx.model,
                "input": {
                    "messages": [{
                        "role": "user",
                        "content": [{ "text": req.prompt }]
                    }]
                },
                "parameters": {
                    "size": image_size(&ctx.model, req.require_aspect_ratio()?),
                    "n": 1,
                    "prompt_extend": true,
                    "watermark": false
                }
            });

            let client = http_client();
            let submit_url = format!("{endpoint}/services/aigc/multimodal-generation/generation");
            let generation_response = client
                .post(&submit_url)
                .bearer_auth(secret)
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?;

            let status = generation_response.status();
            let payload: ResponsePayload =
                response::read_json(generation_response, response::MAX_CONTROL_JSON_BYTES).await?;

            if !status.is_success() {
                let code = payload.code.as_deref().unwrap_or("unknown");
                let msg = payload.message.as_deref().unwrap_or("request failed");
                return Err(AppError::new(
                    "provider_request_failed",
                    format!("DashScope image generation failed: {code} - {msg}"),
                ));
            }

            let (url, revised_prompt) = extract_image(payload.output).ok_or_else(|| {
                AppError::new(
                    "invalid_response",
                    "DashScope image response did not contain an image URL",
                )
            })?;

            let image_bytes = response::download_image(&url).await?;

            Ok(GeneratedImage {
                bytes: image_bytes,
                provider: ctx.provider.clone(),
                model: ctx.model.clone(),
                revised_prompt,
            })
        })
    }
}

fn extract_image(output: OutputPayload) -> Option<(String, Option<String>)> {
    if let Some(image) = output
        .artifacts
        .and_then(|artifacts| artifacts.into_iter().next())
    {
        return Some((image.url, image.actual_prompt));
    }
    output
        .choices?
        .into_iter()
        .flat_map(|c| c.message.content)
        .find_map(|c| c.image.map(|url| (url, None)))
}

/// qwen-image-max/plus only accept five fixed sizes; the other qwen-image
/// models take any size inside their documented range, so the request keeps
/// the exact frame.
fn image_size(model: &str, ratio: &str) -> &'static str {
    if model.starts_with("qwen-image-max") || model.starts_with("qwen-image-plus") {
        return match ratio {
            "3:2" | "16:9" => "1664*928",
            "4:3" => "1472*1104",
            "3:4" => "1104*1472",
            "2:3" | "9:16" => "928*1664",
            _ => "1328*1328",
        };
    }
    match ratio {
        "3:2" | "4:3" => "1536*1024",
        "2:3" | "3:4" => "1024*1536",
        "16:9" => "1920*1080",
        "9:16" => "1080*1920",
        _ => "1328*1328",
    }
}

use futures::future::BoxFuture;
use serde::Deserialize;

use super::endpoint;
use crate::ai::{
    contracts::{GeneratedImage, ImageGenerationCapability, ImageGenerationInput},
    transport::{http_client, response, SecurityContext},
};
use crate::app_error::{AppError, AppResult};

pub struct DashScopeImageAdapter;

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
    fn generate<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ImageGenerationInput,
    ) -> BoxFuture<'a, AppResult<GeneratedImage>> {
        Box::pin(async move {
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
                    "size": image_size(&req.aspect_ratio),
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

            let download_response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?
                .error_for_status()
                .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?;

            let image_bytes =
                response::read_bytes(download_response, response::MAX_IMAGE_BYTES).await?;

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

fn image_size(ratio: &str) -> &'static str {
    match ratio {
        "3:2" => "1536*1024",
        "2:3" => "1024*1536",
        "16:9" => "1920*1080",
        "9:16" => "1080*1920",
        _ => "2048*2048",
    }
}

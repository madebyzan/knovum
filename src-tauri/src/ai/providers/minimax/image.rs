use futures::future::BoxFuture;
use serde::Deserialize;

use super::endpoint;
use crate::ai::{
    contracts::{GeneratedImage, ImageGenerationCapability, ImageGenerationInput},
    transport::{http_client, response, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};

pub struct MiniMaxImageAdapter;

#[derive(Deserialize)]
struct MiniMaxImageResponse {
    output: Option<ImageOutput>,
    data: Option<Vec<GeneratedImageSource>>,
    #[serde(rename = "base_resp")]
    base_response: Option<BaseResponse>,
}

#[derive(Deserialize)]
struct ImageOutput {
    image_urls: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct GeneratedImageSource {
    url: Option<String>,
}

#[derive(Deserialize)]
struct BaseResponse {
    status_code: Option<i64>,
    status_msg: Option<String>,
}

impl ImageGenerationCapability for MiniMaxImageAdapter {
    fn generate<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ImageGenerationInput,
        _cancellation: &'a tokio_util::sync::CancellationToken,
    ) -> BoxFuture<'a, AppResult<GeneratedImage>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let endpoint = endpoint(&ctx.endpoint);
            let client = http_client();

            let model = if ctx.model.is_empty() {
                "image-01"
            } else {
                ctx.model.as_str()
            };

            let ratio = match req.require_aspect_ratio()? {
                requested @ ("16:9" | "9:16" | "4:3" | "3:4") => requested,
                _ => "1:1",
            };

            let body = serde_json::json!({
                "model": model,
                "prompt": req.prompt,
                "aspect_ratio": ratio,
                "response_format": "url",
            });

            let generation_response = client
                .post(format!("{endpoint}/image_generation"))
                .bearer_auth(secret)
                .json(&body)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let generation_response: MiniMaxImageResponse =
                response::read_json(generation_response, response::MAX_CONTROL_JSON_BYTES).await?;

            if let Some(base) = generation_response.base_response {
                if base.status_code.unwrap_or(0) != 0 {
                    let msg = base
                        .status_msg
                        .unwrap_or_else(|| "Unknown error".to_string());
                    return Err(AppError::new("provider_request_failed", msg));
                }
            }

            let image_url = generation_response
                .output
                .and_then(|out| out.image_urls)
                .and_then(|mut urls| urls.pop())
                .or_else(|| {
                    generation_response
                        .data
                        .and_then(|mut images| images.pop())
                        .and_then(|item| item.url)
                })
                .ok_or_else(|| {
                    AppError::new("invalid_response", "MiniMax returned no image URL")
                })?;

            let image_bytes = response::download_image(&image_url).await?;

            Ok(GeneratedImage {
                bytes: image_bytes,
                provider: "minimax".to_string(),
                model: model.to_string(),
                revised_prompt: None,
            })
        })
    }
}

fn provider_err(e: impl std::fmt::Display) -> AppError {
    AppError::new("provider_request_failed", e.to_string())
}

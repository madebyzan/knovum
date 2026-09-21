use futures::future::BoxFuture;
use serde::Deserialize;

use crate::ai::{
    contracts::{GeneratedImage, ImageGenerationCapability, ImageGenerationInput},
    error::{AiError, AiErrorCode, AiResult},
    transport::{http_client, SecurityContext},
};

pub struct MiniMaxImageAdapter;

#[derive(Deserialize)]
struct MiniMaxImageResponse {
    output: Option<ImageOutput>,
    data: Option<Vec<ImageData>>,
    #[serde(rename = "base_resp")]
    base_response: Option<BaseResponse>,
}

#[derive(Deserialize)]
struct ImageOutput {
    image_urls: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ImageData {
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
    ) -> BoxFuture<'a, AiResult<GeneratedImage>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let base = ctx.endpoint.trim_end_matches('/');
            let endpoint = if base.ends_with("/v1") {
                base.to_string()
            } else {
                format!("{base}/v1")
            };
            let client = http_client();

            let model = if ctx.model.is_empty() {
                "image-01"
            } else {
                ctx.model.as_str()
            };

            let ratio = match req.aspect_ratio.as_str() {
                "16:9" | "9:16" | "4:3" | "3:4" => req.aspect_ratio.as_str(),
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
                .map_err(provider_err)?
                .json::<MiniMaxImageResponse>()
                .await
                .map_err(provider_err)?;

            if let Some(base) = generation_response.base_response {
                if base.status_code.unwrap_or(0) != 0 {
                    let msg = base
                        .status_msg
                        .unwrap_or_else(|| "Unknown error".to_string());
                    return Err(AiError::new(AiErrorCode::ProviderRequestFailed, msg));
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
                    AiError::new(
                        AiErrorCode::InvalidResponse,
                        "MiniMax returned no image URL",
                    )
                })?;

            let download_response = client
                .get(&image_url)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;

            let image_bytes = download_response
                .bytes()
                .await
                .map_err(provider_err)?
                .to_vec();

            Ok(GeneratedImage {
                bytes: image_bytes,
                provider: "minimax".to_string(),
                model: model.to_string(),
                revised_prompt: None,
            })
        })
    }
}

fn provider_err(e: impl std::fmt::Display) -> AiError {
    AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string())
}

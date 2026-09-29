use crate::ai::transport::response;
use futures::future::BoxFuture;

use super::image_task;
use crate::ai::{
    contracts::{GeneratedImage, ImageGenerationCapability, ImageGenerationInput},
    error::{AiError, AiErrorCode, AiResult},
    transport::{http_client, SecurityContext},
};

pub struct KlingImageAdapter;

impl ImageGenerationCapability for KlingImageAdapter {
    fn generate<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ImageGenerationInput,
    ) -> BoxFuture<'a, AiResult<GeneratedImage>> {
        Box::pin(async move {
            let api_key = ctx.require_api_key()?;
            let model = if ctx.model.is_empty() {
                "kling-v1"
            } else {
                ctx.model.as_str()
            };

            let aspect_ratio = match req.aspect_ratio.as_str() {
                "1:1" | "9:16" | "16:9" | "4:3" | "3:4" | "3:2" | "2:3" => {
                    req.aspect_ratio.as_str()
                }
                _ => "16:9",
            };

            let endpoint = ctx.endpoint.trim_end_matches('/');
            let submit_url = format!("{endpoint}/v1/images/generations");
            let client = http_client();

            let payload = serde_json::json!({
                "model": model,
                "prompt": req.prompt,
                "aspect_ratio": aspect_ratio,
                "n": 1,
            });

            let submission_response = client
                .post(&submit_url)
                .bearer_auth(api_key)
                .json(&payload)
                .send()
                .await
                .map_err(|e| AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string()))?;

            if !submission_response.status().is_success() {
                let error_text = response::read_text(submission_response).await?;
                return Err(AiError::new(AiErrorCode::ProviderRequestFailed, error_text));
            }

            let submission_data: serde_json::Value =
                response::read_json(submission_response, response::MAX_CONTROL_JSON_BYTES).await?;

            let task_id = submission_data["data"]["task_id"]
                .as_str()
                .ok_or_else(|| {
                    AiError::new(
                        AiErrorCode::InvalidResponse,
                        "Missing task_id in Kling image generation response",
                    )
                })?
                .to_string();

            let image_bytes = image_task::poll(ctx, "generations", &task_id).await?;
            Ok(GeneratedImage {
                bytes: image_bytes,
                provider: "kling".to_string(),
                model: model.to_string(),
                revised_prompt: None,
            })
        })
    }
}

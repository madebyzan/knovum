use crate::ai::transport::response;
use futures::future::BoxFuture;
use std::time::{Duration, Instant};

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

            let query_url = format!("{endpoint}/v1/images/generations/{task_id}");
            let started_at = Instant::now();
            let timeout = Duration::from_secs(180);
            let mut poll_interval = Duration::from_secs(2);

            loop {
                if started_at.elapsed() >= timeout {
                    return Err(AiError::new(
                        AiErrorCode::ProviderRequestFailed,
                        format!("Kling image generation timed out (task_id: {task_id})"),
                    ));
                }

                tokio::time::sleep(poll_interval).await;
                if poll_interval < Duration::from_secs(5) {
                    poll_interval += Duration::from_secs(1);
                }

                let query_response = client
                    .get(&query_url)
                    .bearer_auth(api_key)
                    .send()
                    .await
                    .map_err(|e| AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string()))?;

                if !query_response.status().is_success() {
                    let error_text = response::read_text(query_response).await?;
                    return Err(AiError::new(AiErrorCode::ProviderRequestFailed, error_text));
                }

                let query_data: serde_json::Value =
                    response::read_json(query_response, response::MAX_CONTROL_JSON_BYTES).await?;

                let status = query_data["data"]["task_status"]
                    .as_str()
                    .unwrap_or_default();

                match status {
                    "succeed" => {
                        let image_url = query_data["data"]["task_result"]["images"][0]["url"]
                            .as_str()
                            .ok_or_else(|| {
                                AiError::new(
                                    AiErrorCode::InvalidResponse,
                                    "Missing image URL in Kling response",
                                )
                            })?;

                        let download_response = client
                            .get(image_url)
                            .send()
                            .await
                            .map_err(|e| {
                                AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string())
                            })?
                            .error_for_status()
                            .map_err(|e| {
                                AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string())
                            })?;

                        let image_bytes =
                            response::read_bytes(download_response, response::MAX_IMAGE_BYTES)
                                .await?;

                        return Ok(GeneratedImage {
                            bytes: image_bytes,
                            provider: "kling".to_string(),
                            model: model.to_string(),
                            revised_prompt: None,
                        });
                    }
                    "failed" => {
                        let msg = query_data["data"]["task_status_msg"]
                            .as_str()
                            .unwrap_or("Kling image generation task failed");
                        return Err(AiError::new(AiErrorCode::ProviderRequestFailed, msg));
                    }
                    _ => continue,
                }
            }
        })
    }
}

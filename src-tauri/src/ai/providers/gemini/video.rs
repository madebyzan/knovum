use crate::ai::transport::response;
use futures::future::BoxFuture;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus, VideoGenerationSubmission,
    },
    transport::{http_client, SecurityContext},
};
use crate::app_error::{AppError, AppResult};

pub struct GeminiVideoAdapter;

impl VideoGenerationCapability for GeminiVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let api_key = ctx.require_api_key()?;
            let model = if ctx.model.is_empty() {
                "veo-2.0-generate-001"
            } else {
                ctx.model.as_str()
            };

            let ratio = match req.ratio.as_str() {
                "9:16" | "portrait" => "9:16",
                "1:1" | "square" => "1:1",
                _ => "16:9",
            };

            let mut instance = serde_json::json!({
                "prompt": req.prompt,
                "aspectRatio": ratio,
            });
            if let Some(first_img) = req.reference_images.first() {
                let pure_payload = first_img
                    .split_once(";base64,")
                    .map(|(_, b)| b)
                    .unwrap_or(first_img);
                instance["image"] = serde_json::json!({ "bytesBase64Encoded": pure_payload });
            }

            let payload = serde_json::json!({
                "instances": [instance],
                "parameters": {
                    "sampleCount": 1,
                    "durationSeconds": req.duration.clamp(5, 8),
                }
            });

            let endpoint = ctx.endpoint.trim_end_matches('/');
            let submit_url = format!("{endpoint}/models/{model}:predictLongRunning");
            let client = http_client();

            let submission_response = client
                .post(&submit_url)
                .header("x-goog-api-key", api_key)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?;

            if !submission_response.status().is_success() {
                let error_text = response::read_text(submission_response).await?;
                return Err(AppError::new("provider_request_failed", error_text));
            }

            let initial_payload: serde_json::Value =
                response::read_json(submission_response, response::MAX_CONTROL_JSON_BYTES).await?;

            let operation_name = initial_payload["name"]
                .as_str()
                .ok_or_else(|| AppError::new("invalid_response", "Missing operation name"))?
                .to_string();

            Ok(VideoGenerationSubmission {
                operation_id: operation_name,
                provider: "gemini".to_string(),
                model: model.to_string(),
            })
        })
    }

    fn status<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        task_id: &'a str,
    ) -> BoxFuture<'a, AppResult<VideoGenerationStatus>> {
        Box::pin(async move {
            let api_key = ctx.require_api_key()?;
            let endpoint = ctx.endpoint.trim_end_matches('/');
            let clean_id = task_id.trim_start_matches('/');
            let poll_url = if clean_id.starts_with("http://") || clean_id.starts_with("https://") {
                clean_id.to_string()
            } else if let Some(stripped) = clean_id.strip_prefix("v1beta/") {
                format!("{endpoint}/{stripped}")
            } else {
                format!("{endpoint}/{clean_id}")
            };
            let client = http_client();

            let poll_response = client
                .get(&poll_url)
                .header("x-goog-api-key", api_key)
                .send()
                .await
                .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?;

            if !poll_response.status().is_success() {
                let error_text = response::read_text(poll_response).await?;
                return Err(AppError::new("provider_request_failed", error_text));
            }

            let operation_status: serde_json::Value =
                response::read_json(poll_response, response::MAX_CONTROL_JSON_BYTES).await?;

            if let Some(err) = operation_status["error"]["message"].as_str() {
                return Ok(VideoGenerationStatus::Failed {
                    error_message: err.to_string(),
                });
            }

            if operation_status["done"].as_bool().unwrap_or(false) {
                let sample = &operation_status["response"]["generateVideoResponse"]
                    ["generatedSamples"][0]["video"];
                let video_url = if let Some(b64) = sample["bytesBase64Encoded"].as_str() {
                    format!("data:video/mp4;base64,{b64}")
                } else if let Some(uri) = sample["uri"].as_str() {
                    uri.to_string()
                } else {
                    return Err(AppError::new(
                        "invalid_response",
                        "No video content in Veo response",
                    ));
                };
                return Ok(VideoGenerationStatus::Succeeded { video_url });
            }

            Ok(VideoGenerationStatus::Processing { progress_pct: None })
        })
    }

    fn generate<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
        cancellation: &'a CancellationToken,
        stage: VideoGenerationStageCallback<'a>,
    ) -> BoxFuture<'a, AppResult<GeneratedVideo>> {
        Box::pin(async move {
            crate::ai::capabilities::video::materialize(self, ctx, req, cancellation, stage).await
        })
    }
}

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

pub struct KlingVideoAdapter;

impl VideoGenerationCapability for KlingVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let api_key = ctx.require_api_key()?;
            let model = if ctx.model.is_empty() {
                "kling-v1"
            } else {
                ctx.model.as_str()
            };

            let ratio = match req.ratio.as_str() {
                "9:16" | "portrait" => "9:16",
                "1:1" | "square" => "1:1",
                _ => "16:9",
            };
            let duration_str = if req.duration >= 10 { "10" } else { "5" };

            let has_image = req.reference_images.first();
            let action = if has_image.is_some() {
                "image2video"
            } else {
                "text2video"
            };

            let mut payload = serde_json::json!({
                "model": model,
                "prompt": req.prompt,
                "duration": duration_str,
            });
            if let Some(img) = has_image {
                let pure_payload = img.split_once(";base64,").map(|(_, b)| b).unwrap_or(img);
                payload["image"] = serde_json::json!(pure_payload);
            } else {
                payload["aspect_ratio"] = serde_json::json!(ratio);
            }

            let endpoint = ctx.endpoint.trim_end_matches('/');
            let submit_url = format!("{endpoint}/v1/videos/{action}");
            let client = http_client();

            let submission_response = client
                .post(&submit_url)
                .bearer_auth(api_key)
                .json(&payload)
                .send()
                .await
                .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?;

            if !submission_response.status().is_success() {
                let error_text = response::read_text(submission_response).await?;
                return Err(AppError::new("provider_request_failed", error_text));
            }

            let init_val: serde_json::Value =
                response::read_json(submission_response, response::MAX_CONTROL_JSON_BYTES).await?;

            let raw_task_id = init_val["data"]["task_id"]
                .as_str()
                .ok_or_else(|| AppError::new("invalid_response", "Missing Kling task_id"))?;
            let task_id = format!("{action}/{raw_task_id}");

            Ok(VideoGenerationSubmission {
                operation_id: task_id,
                provider: "kling".to_string(),
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
            let client = http_client();

            let query_path = if task_id.contains('/') {
                task_id.to_string()
            } else {
                format!("text2video/{task_id}")
            };
            let query_url = format!("{endpoint}/v1/videos/{query_path}");
            let query_response = client
                .get(&query_url)
                .bearer_auth(api_key)
                .send()
                .await
                .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?
                .error_for_status()
                .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?;

            let payload: serde_json::Value =
                response::read_json(query_response, response::MAX_CONTROL_JSON_BYTES).await?;

            let status = payload["data"]["task_status"].as_str().unwrap_or_default();
            match status {
                "succeed" => {
                    let video_url = payload["data"]["task_result"]["videos"][0]["url"]
                        .as_str()
                        .ok_or_else(|| {
                            AppError::new("invalid_response", "No video URL in Kling response")
                        })?
                        .to_string();
                    Ok(VideoGenerationStatus::Succeeded { video_url })
                }
                "failed" => {
                    let message = payload["data"]["task_status_msg"]
                        .as_str()
                        .unwrap_or("Kling generation failed")
                        .to_string();
                    Ok(VideoGenerationStatus::Failed {
                        error_message: message,
                    })
                }
                "submitted" => Ok(VideoGenerationStatus::Pending { progress_pct: None }),
                _ => Ok(VideoGenerationStatus::Processing { progress_pct: None }),
            }
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

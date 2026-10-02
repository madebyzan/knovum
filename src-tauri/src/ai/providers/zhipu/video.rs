use futures::future::BoxFuture;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus, VideoGenerationSubmission,
    },
    transport::{http_client, response, SecurityContext},
};
use crate::app_error::{AppError, AppResult};

pub struct ZhipuVideoAdapter;

impl VideoGenerationCapability for ZhipuVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let api_key = ctx.require_api_key()?;
            let model = if ctx.model.is_empty() {
                "cogvideox"
            } else {
                ctx.model.as_str()
            };

            let mut payload = serde_json::json!({
                "model": model,
                "prompt": req.prompt,
            });
            if let Some(img) = req.reference_images.first() {
                payload["image_url"] = serde_json::json!(img);
            }

            let endpoint = zhipu_endpoint(&ctx.endpoint);
            let submit_url = format!("{endpoint}/videos/generations");
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

            let task_id = init_val["id"]
                .as_str()
                .ok_or_else(|| AppError::new("invalid_response", "Missing Zhipu task ID"))?
                .to_string();

            Ok(VideoGenerationSubmission {
                operation_id: task_id,
                provider: "zhipu".to_string(),
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
            let endpoint = zhipu_endpoint(&ctx.endpoint);
            let client = http_client();
            let query_url = format!("{endpoint}/async-result/{task_id}");

            let query_response = client
                .get(&query_url)
                .bearer_auth(api_key)
                .send()
                .await
                .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?;

            if !query_response.status().is_success() {
                let error_text = response::read_text(query_response).await?;
                return Err(AppError::new("provider_request_failed", error_text));
            }

            let val: serde_json::Value =
                response::read_json(query_response, response::MAX_CONTROL_JSON_BYTES).await?;

            let status = val["task_status"].as_str().unwrap_or_default();
            match status {
                "SUCCESS" => {
                    let video_url = val["video_result"][0]["url"]
                        .as_str()
                        .ok_or_else(|| AppError::new("invalid_response", "Missing video URL"))?
                        .to_string();
                    Ok(VideoGenerationStatus::Succeeded { video_url })
                }
                "FAIL" => Ok(VideoGenerationStatus::Failed {
                    error_message: "Zhipu video generation failed".to_string(),
                }),
                "PROCESSING" => Ok(VideoGenerationStatus::Processing { progress_pct: None }),
                _ => Ok(VideoGenerationStatus::Pending { progress_pct: None }),
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

fn zhipu_endpoint(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.ends_with("/api/paas/v4") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/api/paas/v4")
    }
}

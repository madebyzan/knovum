use futures::future::BoxFuture;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus, VideoGenerationSubmission,
    },
    transport::{http_client, response, SecurityContext},
};
use crate::app_error::{AppError, AppResult};

pub struct DoubaoVideoAdapter;

#[derive(Deserialize)]
struct TaskSubmitResponse {
    id: Option<String>,
    error: Option<TaskError>,
}
#[derive(Deserialize)]
struct TaskStatusResponse {
    status: Option<String>,
    content: Option<TaskContent>,
    error: Option<TaskError>,
}
#[derive(Deserialize)]
struct TaskContent {
    video_url: Option<String>,
}
#[derive(Deserialize)]
struct TaskError {
    message: Option<String>,
}

impl VideoGenerationCapability for DoubaoVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let endpoint = doubao_endpoint(&ctx.endpoint);
            let client = http_client();

            let model = if ctx.model.is_empty() {
                "doubao-video-generation"
            } else {
                ctx.model.as_str()
            };

            let mut content = vec![serde_json::json!({
                "type": "text",
                "text": req.prompt,
            })];
            if let Some(first_image) = req.reference_images.first() {
                content.push(serde_json::json!({
                    "type": "image_url",
                    "image_url": { "url": first_image }
                }));
            }

            let body = serde_json::json!({
                "model": model,
                "content": content,
            });

            let submission_response = client
                .post(format!("{endpoint}/contents/generations/tasks"))
                .bearer_auth(secret)
                .json(&body)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let submission_response: TaskSubmitResponse =
                response::read_json(submission_response, response::MAX_CONTROL_JSON_BYTES).await?;

            if let Some(err) = submission_response.error {
                let msg = err
                    .message
                    .unwrap_or_else(|| "Task submission failed".to_string());
                return Err(AppError::new("provider_request_failed", msg));
            }

            let task_id = submission_response
                .id
                .ok_or_else(|| AppError::new("invalid_response", "Doubao returned no task ID"))?;

            Ok(VideoGenerationSubmission {
                operation_id: task_id,
                provider: "doubao".to_string(),
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
            let secret = ctx.require_api_key()?;
            let endpoint = doubao_endpoint(&ctx.endpoint);
            let client = http_client();

            let query_response = client
                .get(format!("{endpoint}/contents/generations/tasks/{task_id}"))
                .bearer_auth(secret)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let query_response: TaskStatusResponse =
                response::read_json(query_response, response::MAX_CONTROL_JSON_BYTES).await?;

            if let Some(err) = query_response.error {
                let msg = err
                    .message
                    .unwrap_or_else(|| "Doubao task error".to_string());
                return Ok(VideoGenerationStatus::Failed { error_message: msg });
            }

            let status = query_response.status.as_deref().unwrap_or("UNKNOWN");
            match status {
                "SUCCEEDED" => {
                    let video_url = query_response
                        .content
                        .and_then(|c| c.video_url)
                        .ok_or_else(|| {
                            AppError::new(
                                "invalid_response",
                                "Doubao task succeeded but returned no video URL",
                            )
                        })?;
                    Ok(VideoGenerationStatus::Succeeded { video_url })
                }
                "QUEUED" | "PENDING" => Ok(VideoGenerationStatus::Pending { progress_pct: None }),
                "RUNNING" | "PROCESSING" => {
                    Ok(VideoGenerationStatus::Processing { progress_pct: None })
                }
                _ => Ok(VideoGenerationStatus::Failed {
                    error_message: format!("Doubao task terminated with status: {status}"),
                }),
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

fn doubao_endpoint(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.ends_with("/api/v3") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/api/v3")
    }
}

fn provider_err(e: impl std::fmt::Display) -> AppError {
    AppError::new("provider_request_failed", e.to_string())
}

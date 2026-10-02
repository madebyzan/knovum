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

pub struct SiliconFlowVideoAdapter;

#[derive(Deserialize)]
struct SubmitResponse {
    #[serde(rename = "requestId")]
    request_id: Option<String>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct StatusResponse {
    status: String,
    #[serde(rename = "results")]
    video_payload: Option<VideoPayload>,
    reason: Option<String>,
}

#[derive(Deserialize)]
struct VideoPayload {
    videos: Option<Vec<GeneratedVideoArtifact>>,
}

#[derive(Deserialize)]
struct GeneratedVideoArtifact {
    url: String,
}

impl VideoGenerationCapability for SiliconFlowVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let endpoint = siliconflow_endpoint(&ctx.endpoint);
            let client = http_client();

            let model = if ctx.model.is_empty() {
                "THUDM/CogVideoX-5b"
            } else {
                ctx.model.as_str()
            };

            let mut body = serde_json::json!({
                "model": model,
                "prompt": req.prompt,
            });

            if let Some(first_image) = req.reference_images.first() {
                body["image"] = serde_json::json!(first_image);
            }

            let submission_response = client
                .post(format!("{endpoint}/video/submit"))
                .bearer_auth(secret)
                .json(&body)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let submission_response: SubmitResponse =
                response::read_json(submission_response, response::MAX_CONTROL_JSON_BYTES).await?;

            let task_id = submission_response.request_id.ok_or_else(|| {
                let msg = submission_response
                    .message
                    .unwrap_or_else(|| "SiliconFlow returned no requestId".to_string());
                AppError::new("invalid_response", msg)
            })?;

            Ok(VideoGenerationSubmission {
                operation_id: task_id,
                provider: "siliconflow".to_string(),
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
            let endpoint = siliconflow_endpoint(&ctx.endpoint);
            let client = http_client();

            let query_response = client
                .get(format!("{endpoint}/video/status"))
                .query(&[("requestId", task_id)])
                .bearer_auth(secret)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let query_response: StatusResponse =
                response::read_json(query_response, response::MAX_CONTROL_JSON_BYTES).await?;

            match query_response.status.as_str() {
                "Succeed" => {
                    let video_url = query_response
                        .video_payload
                        .and_then(|payload| payload.videos)
                        .and_then(|mut videos| videos.pop())
                        .map(|item| item.url)
                        .ok_or_else(|| {
                            AppError::new(
                                "invalid_response",
                                "SiliconFlow returned no video url on success",
                            )
                        })?;
                    Ok(VideoGenerationStatus::Succeeded { video_url })
                }
                "Pending" => Ok(VideoGenerationStatus::Pending { progress_pct: None }),
                "Processing" => Ok(VideoGenerationStatus::Processing { progress_pct: None }),
                _ => {
                    let err_msg = query_response
                        .reason
                        .unwrap_or_else(|| "SiliconFlow video task failed".to_string());
                    Ok(VideoGenerationStatus::Failed {
                        error_message: err_msg,
                    })
                }
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

fn siliconflow_endpoint(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        "https://api.siliconflow.cn/v1".to_string()
    } else if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}

fn provider_err(e: impl std::fmt::Display) -> AppError {
    AppError::new("provider_request_failed", e.to_string())
}

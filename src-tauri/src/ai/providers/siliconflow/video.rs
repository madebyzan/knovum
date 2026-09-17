use futures::future::BoxFuture;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus, VideoGenerationSubmission,
    },
    error::{AiError, AiErrorCode, AiResult},
    transport::{http_client, SecurityContext},
};

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
    videos: Option<Vec<VideoItem>>,
}

#[derive(Deserialize)]
struct VideoItem {
    url: String,
}

impl VideoGenerationCapability for SiliconFlowVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AiResult<VideoGenerationSubmission>> {
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
                .map_err(provider_err)?
                .json::<SubmitResponse>()
                .await
                .map_err(provider_err)?;

            let task_id = submission_response.request_id.ok_or_else(|| {
                let msg = submission_response
                    .message
                    .unwrap_or_else(|| "SiliconFlow returned no requestId".to_string());
                AiError::new(AiErrorCode::InvalidResponse, msg)
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
    ) -> BoxFuture<'a, AiResult<VideoGenerationStatus>> {
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
                .map_err(provider_err)?
                .json::<StatusResponse>()
                .await
                .map_err(provider_err)?;

            match query_response.status.as_str() {
                "Succeed" => {
                    let video_url = query_response
                        .video_payload
                        .and_then(|payload| payload.videos)
                        .and_then(|mut items| items.pop())
                        .map(|item| item.url)
                        .ok_or_else(|| {
                            AiError::new(
                                AiErrorCode::InvalidResponse,
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
    ) -> BoxFuture<'a, AiResult<GeneratedVideo>> {
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

fn provider_err(e: impl std::fmt::Display) -> AiError {
    AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string())
}

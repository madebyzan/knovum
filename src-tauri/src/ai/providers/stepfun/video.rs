use crate::ai::transport::response;
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

pub struct StepFunVideoAdapter;

#[derive(Deserialize)]
struct StepVideoSubmitResponse {
    id: Option<String>,
    error: Option<StepError>,
}

#[derive(Deserialize)]
struct StepVideoQueryResponse {
    status: Option<String>,
    video: Option<StepVideoData>,
    error: Option<StepError>,
}

#[derive(Deserialize)]
struct StepVideoData {
    url: Option<String>,
}

#[derive(Deserialize)]
struct StepError {
    message: Option<String>,
}

impl VideoGenerationCapability for StepFunVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AiResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let endpoint = stepfun_endpoint(&ctx.endpoint);
            let client = http_client();

            let model = if ctx.model.is_empty() {
                "step-video-t2v"
            } else {
                ctx.model.as_str()
            };

            let size = match req.ratio.as_str() {
                "9:16" => "720x1280",
                "1:1" => "960x960",
                _ => "1280x720",
            };

            let body = serde_json::json!({
                "model": model,
                "prompt": req.prompt,
                "size": size,
            });

            let submission_response = client
                .post(format!("{endpoint}/videos/generations"))
                .bearer_auth(secret)
                .json(&body)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let submission_response: StepVideoSubmitResponse =
                response::read_json(submission_response, response::MAX_CONTROL_JSON_BYTES).await?;

            if let Some(err) = submission_response.error {
                let msg = err.message.unwrap_or_else(|| "Submit failed".to_string());
                return Err(AiError::new(AiErrorCode::ProviderRequestFailed, msg));
            }

            let task_id = submission_response.id.ok_or_else(|| {
                AiError::new(AiErrorCode::InvalidResponse, "StepFun returned no task ID")
            })?;

            Ok(VideoGenerationSubmission {
                operation_id: task_id,
                provider: "stepfun".to_string(),
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
            let endpoint = stepfun_endpoint(&ctx.endpoint);
            let client = http_client();

            let query_response = client
                .get(format!("{endpoint}/videos/generations/{task_id}"))
                .bearer_auth(secret)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let query_response: StepVideoQueryResponse =
                response::read_json(query_response, response::MAX_CONTROL_JSON_BYTES).await?;

            if let Some(err) = query_response.error {
                let msg = err.message.unwrap_or_else(|| "Query failed".to_string());
                return Ok(VideoGenerationStatus::Failed { error_message: msg });
            }

            let status = query_response.status.as_deref().unwrap_or("unknown");
            match status {
                "completed" | "succeeded" => {
                    let video_url = query_response.video.and_then(|v| v.url).ok_or_else(|| {
                        AiError::new(
                            AiErrorCode::InvalidResponse,
                            "StepFun completed but returned no video URL",
                        )
                    })?;
                    Ok(VideoGenerationStatus::Succeeded { video_url })
                }
                "processing" | "pending" | "queued" => {
                    Ok(VideoGenerationStatus::Processing { progress_pct: None })
                }
                _ => Ok(VideoGenerationStatus::Failed {
                    error_message: format!("StepFun video task ended with status: {status}"),
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
    ) -> BoxFuture<'a, AiResult<GeneratedVideo>> {
        Box::pin(async move {
            crate::ai::capabilities::video::materialize(self, ctx, req, cancellation, stage).await
        })
    }
}

fn stepfun_endpoint(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}

fn provider_err(e: impl std::fmt::Display) -> AiError {
    AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string())
}

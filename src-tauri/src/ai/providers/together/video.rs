use crate::ai::transport::response;
use futures::future::BoxFuture;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus, VideoGenerationSubmission,
    },
    transport::{http_client, SecurityContext},
};
use crate::app_error::{AppError, AppResult};

pub struct TogetherVideoAdapter;

#[derive(Deserialize)]
struct TogetherVideoSubmitResponse {
    id: Option<String>,
    error: Option<TogetherError>,
}
#[derive(Deserialize)]
struct TogetherVideoQueryResponse {
    status: Option<String>,
    outputs: Option<Vec<TogetherOutput>>,
    error: Option<TogetherError>,
}
#[derive(Deserialize)]
struct TogetherOutput {
    url: Option<String>,
}
#[derive(Deserialize)]
struct TogetherError {
    message: Option<String>,
}

impl VideoGenerationCapability for TogetherVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let endpoint = together_endpoint(&ctx.endpoint);
            let client = http_client();

            let model = if ctx.model.is_empty() {
                "THUDM/CogVideoX-5b"
            } else {
                ctx.model.as_str()
            };

            let (width, height) = match req.ratio.as_str() {
                "9:16" => (720, 1280),
                "1:1" => (960, 960),
                _ => (1280, 720),
            };

            let mut body = serde_json::json!({
                "model": model,
                "prompt": req.prompt,
                "width": width,
                "height": height,
            });

            if let Some(first_img) = req.reference_images.first() {
                body["image_url"] = serde_json::json!(first_img);
            }

            let submission_response = client
                .post(format!("{endpoint}/videos"))
                .bearer_auth(secret)
                .json(&body)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let submission_response: TogetherVideoSubmitResponse =
                response::read_json(submission_response, response::MAX_CONTROL_JSON_BYTES).await?;

            if let Some(err) = submission_response.error {
                let msg = err.message.unwrap_or_else(|| "Submit failed".to_string());
                return Err(AppError::new("provider_request_failed", msg));
            }

            let task_id = submission_response
                .id
                .ok_or_else(|| AppError::new("invalid_response", "Together returned no task ID"))?;

            Ok(VideoGenerationSubmission {
                operation_id: task_id,
                provider: "together".to_string(),
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
            let endpoint = together_endpoint(&ctx.endpoint);
            let client = http_client();

            let query_response = client
                .get(format!("{endpoint}/videos/{task_id}"))
                .bearer_auth(secret)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let query_response: TogetherVideoQueryResponse =
                response::read_json(query_response, response::MAX_CONTROL_JSON_BYTES).await?;

            if let Some(err) = query_response.error {
                let msg = err.message.unwrap_or_else(|| "Query failed".to_string());
                return Ok(VideoGenerationStatus::Failed { error_message: msg });
            }

            let status = query_response.status.as_deref().unwrap_or("unknown");
            match status {
                "completed" | "succeeded" => {
                    let video_url = query_response
                        .outputs
                        .and_then(|mut videos| videos.pop())
                        .and_then(|out| out.url)
                        .ok_or_else(|| {
                            AppError::new(
                                "invalid_response",
                                "Together completed but returned no video URL",
                            )
                        })?;
                    Ok(VideoGenerationStatus::Succeeded { video_url })
                }
                "processing" | "pending" | "queued" => {
                    Ok(VideoGenerationStatus::Processing { progress_pct: None })
                }
                _ => Ok(VideoGenerationStatus::Failed {
                    error_message: format!("Together video task ended with status: {status}"),
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

fn together_endpoint(raw: &str) -> String {
    let base = raw.trim().trim_end_matches('/');
    if base.ends_with("/v1") {
        base.to_string()
    } else {
        format!("{base}/v1")
    }
}

fn provider_err(e: impl std::fmt::Display) -> AppError {
    AppError::new("provider_request_failed", e.to_string())
}

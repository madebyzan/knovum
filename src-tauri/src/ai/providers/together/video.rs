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
use crate::foundation::app_error::{AppError, AppResult};

pub struct TogetherVideoAdapter;

#[derive(Deserialize)]
struct TogetherVideoJob {
    id: Option<String>,
    status: Option<String>,
    outputs: Option<TogetherOutputs>,
    error: Option<TogetherError>,
}

#[derive(Deserialize)]
struct TogetherOutputs {
    video_url: Option<String>,
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
            let endpoint = together_video_endpoint(&ctx.endpoint);
            let client = http_client();
            let model = ctx.model.as_str();

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

            if let Some(first_frame) = req.reference_images.first() {
                body["media"] = serde_json::json!({
                    "frame_images": [{ "input_image": first_frame }],
                });
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
            let job: TogetherVideoJob =
                response::read_json(submission_response, response::MAX_CONTROL_JSON_BYTES).await?;

            if let Some(error) = job.error {
                let message = error.message.unwrap_or_else(|| "Submit failed".to_string());
                return Err(AppError::new("provider_request_failed", message));
            }

            let task_id = job
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
            let endpoint = together_video_endpoint(&ctx.endpoint);
            let client = http_client();

            let query_response = client
                .get(format!("{endpoint}/videos/{task_id}"))
                .bearer_auth(secret)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let job: TogetherVideoJob =
                response::read_json(query_response, response::MAX_CONTROL_JSON_BYTES).await?;

            match job.status.as_deref().unwrap_or_default() {
                "completed" => {
                    let video_url = job
                        .outputs
                        .and_then(|outputs| outputs.video_url)
                        .ok_or_else(|| {
                            AppError::new(
                                "invalid_response",
                                "Together completed but returned no video URL",
                            )
                        })?;
                    Ok(VideoGenerationStatus::Succeeded { video_url })
                }
                "queued" => Ok(VideoGenerationStatus::Pending { progress_pct: None }),
                "in_progress" => Ok(VideoGenerationStatus::Processing { progress_pct: None }),
                "failed" | "cancelled" => {
                    let message = job
                        .error
                        .and_then(|error| error.message)
                        .unwrap_or_else(|| {
                            format!("Together video task {}", job.status.unwrap_or_default())
                        });
                    Ok(VideoGenerationStatus::Failed {
                        error_message: message,
                    })
                }
                unknown => Err(AppError::new(
                    "invalid_response",
                    format!("Together returned an unknown video task status: {unknown}"),
                )),
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

/// Together serves chat and images from `/v1` but video jobs from `/v2`.
fn together_video_endpoint(raw: &str) -> String {
    let base = raw.trim().trim_end_matches('/');
    let root = base
        .strip_suffix("/v2")
        .or_else(|| base.strip_suffix("/v1"))
        .unwrap_or(base);
    format!("{root}/v2")
}

fn provider_err(e: impl std::fmt::Display) -> AppError {
    AppError::new("provider_request_failed", e.to_string())
}

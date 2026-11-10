use futures::future::BoxFuture;
use serde::Deserialize;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::endpoint;
use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus, VideoGenerationSubmission,
    },
    transport::{http_client, response, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};

/// Hy-Video on TokenHub: an async task that answers with a temporary signed
/// COS URL, so the result needs no extra download headers.
pub struct HunyuanVideoAdapter;

#[derive(Deserialize)]
struct SubmissionResponse {
    #[serde(default)]
    task_id: String,
    #[serde(default)]
    error: Option<JobError>,
}

#[derive(Deserialize)]
struct JobStatusResponse {
    status: String,
    #[serde(default)]
    videos: Option<Vec<VideoPayload>>,
    #[serde(default)]
    error: Option<JobError>,
}

#[derive(Deserialize)]
struct VideoPayload {
    url: String,
}

#[derive(Deserialize)]
struct JobError {
    #[serde(default)]
    message: Option<String>,
}

impl VideoGenerationCapability for HunyuanVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let mut body = json!({ "model": ctx.model, "prompt": req.prompt });
            match req.reference_images.first() {
                // Carrying a picture switches the job to image-to-video and
                // the picture then decides the frame, like the API specifies.
                // TokenHub splits the two shapes into separate fields: a
                // public URL goes to `image_url`, inline pixels go to `image`
                // as bare base64 without the data-URI prefix.
                Some(reference) if reference.starts_with("http://") || reference.starts_with("https://") => {
                    body["image_url"] = json!(reference);
                }
                Some(reference) => {
                    let base64 = reference
                        .split_once(";base64,")
                        .map_or(reference.as_str(), |(_, payload)| payload);
                    body["image"] = json!(base64);
                }
                None => body["aspect_ratio"] = json!(req.ratio),
            };

            let endpoint = endpoint(&ctx.endpoint);
            let submission = http_client()
                .post(format!("{endpoint}/wand/hunyuan-video/generation"))
                .bearer_auth(secret)
                .json(&body)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let submission: SubmissionResponse =
                response::read_json(submission, response::MAX_CONTROL_JSON_BYTES).await?;
            // A rejected submission still answers 200 with an empty task id.
            if submission.task_id.is_empty() {
                return Err(submission.error.map(job_error).unwrap_or_else(|| {
                    parse_err("TokenHub video submission returned no task id")
                }));
            }

            Ok(VideoGenerationSubmission {
                operation_id: submission.task_id,
                provider: ctx.provider.clone(),
                model: ctx.model.clone(),
            })
        })
    }

    fn status<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        operation_id: &'a str,
    ) -> BoxFuture<'a, AppResult<VideoGenerationStatus>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let endpoint = endpoint(&ctx.endpoint);
            let query = http_client()
                .get(format!(
                    "{endpoint}/wand/hunyuan-video/tasks/{operation_id}"
                ))
                .bearer_auth(secret)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let job: JobStatusResponse =
                response::read_json(query, response::MAX_CONTROL_JSON_BYTES).await?;
            parse_status(job)
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

fn parse_status(job: JobStatusResponse) -> AppResult<VideoGenerationStatus> {
    match job.status.as_str() {
        "queued" => Ok(VideoGenerationStatus::Pending { progress_pct: None }),
        "running" => Ok(VideoGenerationStatus::Processing { progress_pct: None }),
        "succeeded" => {
            let video = job
                .videos
                .and_then(|mut videos| videos.pop())
                .ok_or_else(|| {
                    AppError::new(
                        "invalid_response",
                        "TokenHub video job finished without a URL",
                    )
                })?;
            Ok(VideoGenerationStatus::Succeeded {
                video_url: video.url,
            })
        }
        "failed" => {
            let error_message = job
                .error
                .and_then(|error| error.message)
                .unwrap_or_else(|| "TokenHub video generation failed".to_string());
            Ok(VideoGenerationStatus::Failed { error_message })
        }
        other => Err(AppError::new(
            "invalid_response",
            format!("TokenHub returned an unknown video status: {other}"),
        )),
    }
}

fn job_error(error: JobError) -> AppError {
    AppError::new(
        "provider_request_failed",
        error
            .message
            .unwrap_or_else(|| "TokenHub video submission failed".to_string()),
    )
}

fn parse_err(message: impl Into<String>) -> AppError {
    AppError::new("invalid_response", message)
}

fn provider_err(error: impl std::fmt::Display) -> AppError {
    AppError::new("provider_request_failed", error.to_string())
}

#[cfg(test)]
#[path = "tests/video.rs"]
mod tests;

use futures::future::BoxFuture;
use serde::Deserialize;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus, VideoGenerationSubmission,
    },
    transport::{http_client, response, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};

pub struct GrokVideoAdapter;

#[derive(Deserialize)]
struct SubmissionResponse {
    request_id: String,
}

#[derive(Deserialize)]
struct JobStatusResponse {
    status: String,
    #[serde(default)]
    video: Option<VideoPayload>,
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

impl VideoGenerationCapability for GrokVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let mut body = json!({
                "model": ctx.model,
                "prompt": req.prompt,
                "duration": req.duration_seconds,
            });
            match req.reference_images.first() {
                // The image field takes a public URL, a base64 data URI, or a
                // Files API id; Knovum references arrive as data URIs.
                Some(reference) => body["image"] = json!({ "url": reference }),
                // With a first frame the image decides the shape, so the ratio
                // only applies to text-to-video.
                None => body["aspect_ratio"] = json!(req.ratio),
            };

            let endpoint = ctx.endpoint.trim_end_matches('/');
            let submission = http_client()
                .post(format!("{endpoint}/videos/generations"))
                .bearer_auth(secret)
                .json(&body)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let submission: SubmissionResponse =
                response::read_json(submission, response::MAX_CONTROL_JSON_BYTES).await?;

            Ok(VideoGenerationSubmission {
                operation_id: submission.request_id,
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
            let endpoint = ctx.endpoint.trim_end_matches('/');
            let query = http_client()
                .get(format!("{endpoint}/videos/{operation_id}"))
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
        "pending" => Ok(VideoGenerationStatus::Pending { progress_pct: None }),
        "done" => {
            let video = job.video.ok_or_else(|| {
                AppError::new("invalid_response", "Grok video job finished without a URL")
            })?;
            Ok(VideoGenerationStatus::Succeeded {
                video_url: video.url,
            })
        }
        "failed" | "expired" => {
            let error_message = job
                .error
                .and_then(|error| error.message)
                .unwrap_or_else(|| "Grok video generation failed".to_string());
            Ok(VideoGenerationStatus::Failed { error_message })
        }
        other => Err(AppError::new(
            "invalid_response",
            format!("Grok returned an unknown video status: {other}"),
        )),
    }
}

fn provider_err(e: impl std::fmt::Display) -> AppError {
    AppError::new("provider_request_failed", e.to_string())
}

#[cfg(test)]
#[path = "tests/video.rs"]
mod tests;

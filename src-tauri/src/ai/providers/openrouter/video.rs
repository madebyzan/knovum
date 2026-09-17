use futures::future::BoxFuture;
use reqwest::Method;
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

pub struct OpenRouterVideoAdapter;

#[derive(Deserialize)]
struct VideoJobResponse {
    id: Option<String>,
    job_id: Option<String>,
    status: Option<String>,
    unsigned_urls: Option<Vec<String>>,
    output: Option<VideoOutputData>,
    error: Option<VideoErrorData>,
}

#[derive(Deserialize)]
struct VideoOutputData {
    video_url: Option<String>,
    urls: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct VideoErrorData {
    message: Option<String>,
}

impl VideoGenerationCapability for OpenRouterVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AiResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let url = format!("{}/videos", openrouter_endpoint(&ctx.endpoint));
            let model = if ctx.model.is_empty() {
                "kuaishou/kling-v1"
            } else {
                &ctx.model
            };

            let mut body = serde_json::json!({
                "model": model, "prompt": req.prompt,
                "aspect_ratio": req.ratio, "duration": req.duration,
            });
            if let Some(img) = req.reference_images.first() {
                body["image_url"] = serde_json::json!(img);
            }

            let submission = openrouter_request(Method::POST, url, secret)
                .json(&body)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?
                .json::<VideoJobResponse>()
                .await
                .map_err(provider_err)?;

            if let Some(err) = submission.error {
                let msg = err.message.unwrap_or_else(|| "Submit failed".to_string());
                return Err(AiError::new(AiErrorCode::ProviderRequestFailed, msg));
            }

            let task_id = submission
                .id
                .or(submission.job_id)
                .ok_or_else(|| AiError::new(AiErrorCode::InvalidResponse, "No job ID returned"))?;

            Ok(VideoGenerationSubmission {
                operation_id: task_id,
                provider: "openrouter".to_string(),
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
            let url = format!("{}/videos/{task_id}", openrouter_endpoint(&ctx.endpoint));

            let query = openrouter_request(Method::GET, url, secret)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?
                .json::<VideoJobResponse>()
                .await
                .map_err(provider_err)?;

            if let Some(err) = query.error {
                let msg = err.message.unwrap_or_else(|| "Job failed".to_string());
                return Ok(VideoGenerationStatus::Failed { error_message: msg });
            }

            let status = query.status.as_deref().unwrap_or("unknown");
            match status {
                "completed" | "succeeded" => {
                    let video_url = extract_video_url(query).ok_or_else(|| {
                        AiError::new(AiErrorCode::InvalidResponse, "No video URL returned")
                    })?;
                    Ok(VideoGenerationStatus::Succeeded { video_url })
                }
                "pending" | "processing" | "queued" | "in_progress" => {
                    Ok(VideoGenerationStatus::Processing { progress_pct: None })
                }
                _ => Ok(VideoGenerationStatus::Failed {
                    error_message: format!("Video job ended with status: {status}"),
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

fn openrouter_request(method: Method, url: String, secret: &str) -> reqwest::RequestBuilder {
    http_client()
        .request(method, url)
        .bearer_auth(secret)
        .header("HTTP-Referer", "https://knovum.space")
        .header("X-Title", "Knovum")
}

fn extract_video_url(job: VideoJobResponse) -> Option<String> {
    job.unsigned_urls
        .and_then(|mut u| u.drain(..).next())
        .or_else(|| {
            job.output.and_then(|o| {
                o.video_url
                    .or_else(|| o.urls.and_then(|mut u| u.drain(..).next()))
            })
        })
}

fn openrouter_endpoint(raw: &str) -> String {
    let t = raw.trim().trim_end_matches('/');
    if t.ends_with("/api/v1") || t.ends_with("/v1") {
        t.to_string()
    } else {
        format!("{t}/api/v1")
    }
}

fn provider_err(e: impl std::fmt::Display) -> AiError {
    AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string())
}

use futures::future::BoxFuture;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Method;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

use super::endpoint;
use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus, VideoGenerationSubmission,
    },
    transport::{http_client, response, validate_endpoint, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};
use crate::foundation::public_network::same_origin;

pub struct OpenRouterVideoAdapter;

#[derive(Deserialize)]
struct SubmissionResponse {
    id: String,
}

#[derive(Deserialize)]
struct JobStatusResponse {
    status: String,
    #[serde(default)]
    unsigned_urls: Vec<String>,
    #[serde(default)]
    urls: Vec<String>,
    #[serde(default)]
    error: Option<Value>,
}

impl VideoGenerationCapability for OpenRouterVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let url = format!("{}/videos", endpoint(&ctx.endpoint));
            let mut body = json!({
                "model": ctx.model,
                "prompt": req.prompt,
                "duration": req.duration_seconds,
                "aspect_ratio": req.ratio,
            });
            if let Some(reference) = req.reference_images.first() {
                // A pinned first frame takes precedence over loose references.
                body["frame_images"] = json!([{
                    "type": "image_url",
                    "image_url": { "url": reference },
                    "frame_type": "first_frame",
                }]);
            }

            let submission = openrouter_request(Method::POST, url, secret)
                .json(&body)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?;
            let submission: SubmissionResponse =
                response::read_json(submission, response::MAX_CONTROL_JSON_BYTES).await?;

            Ok(VideoGenerationSubmission {
                operation_id: submission.id,
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
            let url = format!("{}/videos/{operation_id}", endpoint(&ctx.endpoint));
            let query = openrouter_request(Method::GET, url, secret)
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

    fn result_download_headers(
        &self,
        ctx: &SecurityContext,
        url: &reqwest::Url,
    ) -> AppResult<HeaderMap> {
        let mut headers = HeaderMap::new();
        // Completed jobs may only expose unsigned OpenRouter content URLs,
        // which are fetched with the same key used for polling.
        if same_origin(url, &validate_endpoint(&ctx.endpoint)?) {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", ctx.require_api_key()?)).map_err(
                    |_| AppError::invalid("OpenRouter API key is not a valid request header"),
                )?,
            );
        }
        Ok(headers)
    }
}

fn parse_status(job: JobStatusResponse) -> AppResult<VideoGenerationStatus> {
    match job.status.as_str() {
        "pending" => Ok(VideoGenerationStatus::Pending { progress_pct: None }),
        "in_progress" => Ok(VideoGenerationStatus::Processing { progress_pct: None }),
        "completed" => {
            let video_url = job
                .unsigned_urls
                .into_iter()
                .chain(job.urls)
                .next()
                .ok_or_else(|| {
                    AppError::new(
                        "invalid_response",
                        "OpenRouter video job completed without an output URL",
                    )
                })?;
            Ok(VideoGenerationStatus::Succeeded { video_url })
        }
        "failed" => {
            let error_message = job
                .error
                .as_ref()
                .and_then(|error| {
                    error
                        .as_str()
                        .or_else(|| error.get("message").and_then(Value::as_str))
                })
                .unwrap_or("OpenRouter video generation failed")
                .to_string();
            Ok(VideoGenerationStatus::Failed { error_message })
        }
        other => Err(AppError::new(
            "invalid_response",
            format!("OpenRouter returned an unknown video status: {other}"),
        )),
    }
}

fn openrouter_request(method: Method, url: String, secret: &str) -> reqwest::RequestBuilder {
    http_client()
        .request(method, url)
        .bearer_auth(secret)
        .header("HTTP-Referer", "https://knovum.space")
        .header("X-Title", "Knovum")
}

fn provider_err(e: impl std::fmt::Display) -> AppError {
    AppError::new("provider_request_failed", e.to_string())
}

#[cfg(test)]
#[path = "tests/video.rs"]
mod tests;

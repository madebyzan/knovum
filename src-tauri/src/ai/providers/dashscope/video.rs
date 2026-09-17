use futures::future::BoxFuture;
use tokio_util::sync::CancellationToken;

use super::{image::dashscope_endpoint, protocol};
use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus, VideoGenerationSubmission,
    },
    error::{AiError, AiErrorCode, AiResult},
    transport::{http_client, SecurityContext},
};

pub struct DashScopeVideoAdapter;

impl VideoGenerationCapability for DashScopeVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AiResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let endpoint = dashscope_endpoint(&ctx.endpoint)?;
            let client = http_client();

            let request_body = protocol::create_wan_request(
                &ctx.model,
                &req.prompt,
                &req.ratio,
                req.duration,
                &req.reference_images,
            );
            let submission_response = client
                .post(format!(
                    "{endpoint}/services/aigc/video-generation/video-synthesis"
                ))
                .bearer_auth(secret)
                .header("X-DashScope-Async", "enable")
                .json(&request_body)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?
                .json::<protocol::WanResponse>()
                .await
                .map_err(provider_err)?;

            let task = submission_response.output.ok_or_else(|| {
                response_error(submission_response.code, submission_response.message)
            })?;

            Ok(VideoGenerationSubmission {
                operation_id: task.task_id,
                provider: "dashscope".to_string(),
                model: ctx.model.clone(),
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
            let endpoint = dashscope_endpoint(&ctx.endpoint)?;
            let client = http_client();

            let query_response = client
                .get(format!("{endpoint}/tasks/{task_id}"))
                .bearer_auth(secret)
                .send()
                .await
                .map_err(provider_err)?
                .error_for_status()
                .map_err(provider_err)?
                .json::<protocol::WanResponse>()
                .await
                .map_err(provider_err)?;

            let output = query_response
                .output
                .ok_or_else(|| response_error(query_response.code, query_response.message))?;

            match output.task_status.as_str() {
                "SUCCEEDED" => {
                    let video_url = output.video_url.ok_or_else(|| {
                        AiError::new(AiErrorCode::EmptyResponse, "Wan returned no video URL")
                    })?;
                    Ok(VideoGenerationStatus::Succeeded { video_url })
                }
                "PENDING" => Ok(VideoGenerationStatus::Pending { progress_pct: None }),
                "RUNNING" => Ok(VideoGenerationStatus::Processing { progress_pct: None }),
                _ => {
                    let err = response_error(output.code, output.message);
                    Ok(VideoGenerationStatus::Failed {
                        error_message: err.to_string(),
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

fn response_error(code: Option<String>, message: Option<String>) -> AiError {
    let code = code.as_deref().unwrap_or("unknown");
    let message = message.as_deref().unwrap_or("request failed");
    AiError::new(
        AiErrorCode::ProviderRequestFailed,
        format!("{code}: {message}"),
    )
}

fn provider_err(e: impl std::fmt::Display) -> AiError {
    AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string())
}

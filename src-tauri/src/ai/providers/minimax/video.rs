use futures::future::BoxFuture;
use tokio_util::sync::CancellationToken;

use super::endpoint;
use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus, VideoGenerationSubmission,
    },
    transport::{http_client, response, SecurityContext},
};
use crate::app_error::{AppError, AppResult};

pub struct MiniMaxVideoAdapter;

impl VideoGenerationCapability for MiniMaxVideoAdapter {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>> {
        Box::pin(async move {
            let api_key = ctx.require_api_key()?;
            let model = if ctx.model.is_empty() {
                "video-01"
            } else {
                ctx.model.as_str()
            };
            let mut payload = serde_json::json!({ "prompt": req.prompt, "model": model });
            if let Some(first_frame) = req.reference_images.first() {
                payload["first_frame_image"] = serde_json::json!(first_frame);
            }

            let endpoint = endpoint(&ctx.endpoint);
            let submission_response = http_client()
                .post(format!("{endpoint}/video_generation"))
                .bearer_auth(api_key)
                .json(&payload)
                .send()
                .await
                .map_err(req_err)?;

            if !submission_response.status().is_success() {
                let error_text = response::read_text(submission_response).await?;
                return Err(AppError::new("provider_request_failed", error_text));
            }

            let submission_data: serde_json::Value =
                response::read_json(submission_response, response::MAX_CONTROL_JSON_BYTES).await?;
            let task_id = submission_data["task_id"]
                .as_str()
                .ok_or_else(|| parse_err("Missing task_id in MiniMax video response"))?
                .to_string();

            Ok(VideoGenerationSubmission {
                operation_id: task_id,
                provider: "minimax".to_string(),
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
            let api_key = ctx.require_api_key()?;
            let endpoint = endpoint(&ctx.endpoint);
            let query_url = format!("{endpoint}/query/video_generation?task_id={task_id}");

            let query_response = http_client()
                .get(&query_url)
                .bearer_auth(api_key)
                .send()
                .await
                .map_err(req_err)?;

            if !query_response.status().is_success() {
                let error_text = response::read_text(query_response).await?;
                return Err(AppError::new("provider_request_failed", error_text));
            }

            let query_data: serde_json::Value =
                response::read_json(query_response, response::MAX_CONTROL_JSON_BYTES).await?;
            match query_data["status"].as_str().unwrap_or_default() {
                "Success" => {
                    if let Some(direct_url) = query_data["video_url"]
                        .as_str()
                        .or_else(|| query_data["download_url"].as_str())
                    {
                        return Ok(VideoGenerationStatus::Succeeded {
                            video_url: direct_url.to_string(),
                        });
                    }

                    let file_id = query_data["file_id"]
                        .as_str()
                        .ok_or_else(|| parse_err("Missing video_url and file_id"))?;
                    let download_url = retrieve_file_url(&endpoint, api_key, file_id).await?;
                    Ok(VideoGenerationStatus::Succeeded {
                        video_url: download_url,
                    })
                }
                "Fail" => {
                    let msg = query_data["base_resp"]["status_msg"]
                        .as_str()
                        .unwrap_or("MiniMax video generation failed");
                    Ok(VideoGenerationStatus::Failed {
                        error_message: msg.to_string(),
                    })
                }
                "Processing" => Ok(VideoGenerationStatus::Processing { progress_pct: None }),
                _ => Ok(VideoGenerationStatus::Pending { progress_pct: None }),
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

fn req_err(e: impl std::fmt::Display) -> AppError {
    AppError::new("provider_request_failed", e.to_string())
}

fn parse_err(e: impl std::fmt::Display) -> AppError {
    AppError::new("invalid_response", e.to_string())
}

async fn retrieve_file_url(endpoint: &str, api_key: &str, file_id: &str) -> AppResult<String> {
    let file_url = format!("{endpoint}/files/retrieve?file_id={file_id}");
    let file_response = http_client()
        .get(&file_url)
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(req_err)?;

    let file_data: serde_json::Value =
        response::read_json(file_response, response::MAX_CONTROL_JSON_BYTES).await?;
    file_data["file"]["download_url"]
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| parse_err("Missing download_url in MiniMax file retrieval"))
}

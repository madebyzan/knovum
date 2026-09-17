use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus,
    },
    error::{AiError, AiErrorCode, AiResult},
    transport::{http_client, SecurityContext},
};

pub async fn materialize(
    adapter: &dyn VideoGenerationCapability,
    ctx: &SecurityContext,
    req: &VideoGenerationInput,
    cancellation: &CancellationToken,
    stage: VideoGenerationStageCallback<'_>,
) -> AiResult<GeneratedVideo> {
    stage("submittingVideo").map_err(AiError::invalid)?;
    let submission = adapter.submit(ctx, req).await?;
    let operation_id = submission.operation_id;
    stage("videoGenerationSubmitted").map_err(AiError::invalid)?;

    let mut interval = Duration::from_secs(5);
    let max_interval = Duration::from_secs(30);
    let started_at = Instant::now();
    let max_duration = Duration::from_secs(3600);

    loop {
        if cancellation.is_cancelled() {
            return Err(AiError::cancelled("Video generation cancelled by user"));
        }
        if started_at.elapsed() >= max_duration {
            return Err(AiError::new(
                AiErrorCode::ProviderRequestFailed,
                format!("Video generation timed out after 60 minutes (operation: {operation_id})"),
            ));
        }

        tokio::select! {
            _ = cancellation.cancelled() => {
                return Err(AiError::cancelled("Video generation cancelled by user"));
            }
            _ = tokio::time::sleep(interval) => {}
        }

        if interval < max_interval {
            interval = (interval + Duration::from_secs(5)).min(max_interval);
        }

        stage("generatingVideo").map_err(AiError::invalid)?;

        let generation_status = adapter.status(ctx, &operation_id).await?;
        match generation_status {
            VideoGenerationStatus::Succeeded { video_url } => {
                stage("downloadingVideo").map_err(AiError::invalid)?;
                let video_bytes = if let Some(b64) =
                    video_url.strip_prefix("data:video/mp4;base64,")
                {
                    use base64::Engine;
                    base64::engine::general_purpose::STANDARD
                        .decode(b64)
                        .map_err(|e| AiError::new(AiErrorCode::InvalidResponse, e.to_string()))?
                } else {
                    let download_response = http_client()
                        .get(&video_url)
                        .send()
                        .await
                        .map_err(|e| {
                            AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string())
                        })?
                        .error_for_status()
                        .map_err(|e| {
                            AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string())
                        })?;

                    download_response
                        .bytes()
                        .await
                        .map_err(|e| AiError::new(AiErrorCode::InvalidResponse, e.to_string()))?
                        .to_vec()
                };

                return Ok(GeneratedVideo {
                    bytes: video_bytes,
                    provider: submission.provider,
                    model: submission.model,
                    operation_id,
                });
            }
            VideoGenerationStatus::Failed { error_message } => {
                return Err(AiError::new(
                    AiErrorCode::ProviderRequestFailed,
                    error_message,
                ));
            }
            VideoGenerationStatus::Pending { .. } | VideoGenerationStatus::Processing { .. } => {
                continue;
            }
        }
    }
}

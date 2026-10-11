use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{
        GeneratedVideo, VideoGenerationCapability, VideoGenerationInput,
        VideoGenerationStageCallback, VideoGenerationStatus,
    },
    transport::SecurityContext,
};
use crate::foundation::app_error::{AppError, AppResult};

pub async fn materialize(
    adapter: &dyn VideoGenerationCapability,
    ctx: &SecurityContext,
    req: &VideoGenerationInput,
    cancellation: &CancellationToken,
    stage: VideoGenerationStageCallback<'_>,
) -> AppResult<GeneratedVideo> {
    stage("submittingVideo").map_err(AppError::invalid)?;
    let submission = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(AppError::cancelled("Video generation cancelled by user")),
        submission = adapter.submit(ctx, req) => submission?,
    };
    let operation_id = submission.operation_id;
    stage("videoGenerationSubmitted").map_err(AppError::invalid)?;

    let mut interval = Duration::from_secs(5);
    let max_interval = Duration::from_secs(30);
    let started_at = Instant::now();
    let max_duration = Duration::from_secs(3600);

    loop {
        if cancellation.is_cancelled() {
            return Err(AppError::cancelled("Video generation cancelled by user"));
        }
        if started_at.elapsed() >= max_duration {
            return Err(AppError::new(
                "provider_request_failed",
                format!("Video generation timed out after 60 minutes (operation: {operation_id})"),
            ));
        }

        tokio::select! {
            _ = cancellation.cancelled() => {
                return Err(AppError::cancelled("Video generation cancelled by user"));
            }
            _ = tokio::time::sleep(interval) => {}
        }

        if interval < max_interval {
            interval = (interval + Duration::from_secs(5)).min(max_interval);
        }

        stage("generatingVideo").map_err(AppError::invalid)?;

        let generation_status = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(AppError::cancelled("Video generation cancelled by user")),
            status = adapter.status(ctx, &operation_id) => status?,
        };
        match generation_status {
            VideoGenerationStatus::Succeeded { video_url } => {
                stage("downloadingVideo").map_err(AppError::invalid)?;
                let video_bytes = super::output::materialize(ctx, &video_url, cancellation).await?;

                return Ok(GeneratedVideo {
                    bytes: video_bytes,
                    provider: submission.provider,
                    model: submission.model,
                    operation_id,
                });
            }
            VideoGenerationStatus::Failed { error_message } => {
                return Err(AppError::new("provider_request_failed", error_message));
            }
            VideoGenerationStatus::Pending { .. } | VideoGenerationStatus::Processing { .. } => {
                continue;
            }
        }
    }
}

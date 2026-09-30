mod media;
mod node;
mod visual;

use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::{
    ai::tasks::TaskTargetSelection, spaces::assets::transcription::MediaTranscriptionProgress,
};

use super::{job::EnrichmentJobType, EnqueueEnrichmentJobRequest};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunEnrichmentJobRequest {
    pub job_id: String,
    pub space_id: String,
    pub node_id: String,
    pub job_type: EnrichmentJobType,
    pub source_revision: String,
    pub vision_target: Option<TaskTargetSelection>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichmentJobOutput {
    pub node_id: String,
    pub data: Value,
}

pub async fn execute_job(
    app: AppHandle,
    request: RunEnrichmentJobRequest,
    progress: Channel<MediaTranscriptionProgress>,
) -> Result<EnrichmentJobOutput, String> {
    let seed = EnqueueEnrichmentJobRequest {
        space_id: request.space_id.clone(),
        node_id: request.node_id.clone(),
        job_type: request.job_type,
        source_revision: request.source_revision.clone(),
    };
    super::validate_request(&seed)?;
    let _permit = lane(request.job_type).await?;
    if !super::claim(&app, &request.job_id, &seed).await? {
        return Err("Enrichment is already running for this node revision".to_owned());
    }
    let job_id = request.job_id.clone();
    let (cancellation, _lease) = match crate::ai::executions::acquire(job_id.clone()) {
        Ok(execution) => execution,
        Err(error) => {
            super::mark_failed(&app, &job_id, error.clone()).await?;
            return Err(error);
        }
    };
    match execute(&app, &job_id, request, progress, &cancellation).await {
        Ok(output) => {
            // A job cancelled mid-run keeps its cancelled row; already committed data remains valid.
            super::mark_succeeded(&app, &job_id).await?;
            Ok(output)
        }
        Err(error) => {
            super::mark_failed(&app, &job_id, error.clone()).await?;
            Err(error)
        }
    }
}

async fn execute(
    app: &AppHandle,
    job_id: &str,
    mut request: RunEnrichmentJobRequest,
    progress: Channel<MediaTranscriptionProgress>,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<EnrichmentJobOutput, String> {
    let mut data = node::load_current(app, &request).await?;
    let expected_media_knowledge = data["mediaKnowledge"].clone();
    match request.job_type {
        EnrichmentJobType::ParseDocument => {
            let data = crate::spaces::assets::enrichment::enrich_node_with_cancellation(
                app,
                request.space_id.clone(),
                request.node_id.clone(),
                cancellation,
                None,
                Some(job_id),
            )
            .await?;
            return Ok(EnrichmentJobOutput {
                node_id: request.node_id,
                data,
            });
        }
        EnrichmentJobType::AnalyzeDocument => {
            let selected = request
                .vision_target
                .take()
                .ok_or("Document visual analysis requires an AI target")?;
            let target = crate::ai::settings::resolve_task(
                app,
                selected,
                crate::ai::contracts::ExecutionCapability::VisionUnderstanding,
            )
            .map_err(|error| error.task_message())?;
            let data = crate::spaces::assets::enrichment::document::execute::analyze(
                app,
                &request,
                data,
                target,
                &progress,
                cancellation,
            )
            .await?;
            return Ok(EnrichmentJobOutput {
                node_id: request.node_id,
                data,
            });
        }
        EnrichmentJobType::AnalyzeVisual => {
            let target = request.vision_target.take().ok_or_else(|| {
                "ai-not-configured: Visual analysis requires an execution choice".to_owned()
            })?;
            let target = crate::ai::settings::resolve_task(
                app,
                target,
                crate::ai::contracts::ExecutionCapability::VisionUnderstanding,
            )
            .map_err(|error| error.task_message())?;
            visual::analyze(app, &request, &mut data, target, cancellation).await?;
        }
        EnrichmentJobType::AnalyzeMedia => {
            media::analyze(app, &request, &mut data, &progress, cancellation).await?;
        }
        EnrichmentJobType::TranscribeMedia => {
            media::transcribe(app, job_id, &request, &mut data, progress).await?;
        }
    }
    let data = node::save(app, &request, data, &expected_media_knowledge, cancellation).await?;
    Ok(EnrichmentJobOutput {
        node_id: request.node_id,
        data,
    })
}

async fn lane(job_type: EnrichmentJobType) -> Result<OwnedSemaphorePermit, String> {
    static ANALYSIS: OnceLock<Arc<Semaphore>> = OnceLock::new();
    static MEDIA: OnceLock<Arc<Semaphore>> = OnceLock::new();
    static VISUAL: OnceLock<Arc<Semaphore>> = OnceLock::new();
    static DOCUMENT: OnceLock<Arc<Semaphore>> = OnceLock::new();
    let semaphore = match job_type {
        EnrichmentJobType::AnalyzeMedia => ANALYSIS.get_or_init(|| Arc::new(Semaphore::new(1))),
        EnrichmentJobType::TranscribeMedia => MEDIA.get_or_init(|| Arc::new(Semaphore::new(1))),
        EnrichmentJobType::ParseDocument => DOCUMENT.get_or_init(|| Arc::new(Semaphore::new(1))),
        _ => VISUAL.get_or_init(|| Arc::new(Semaphore::new(1))),
    };
    semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| "Enrichment lane is unavailable".to_owned())
}

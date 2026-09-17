mod node;
mod visual;

use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::{
    ai::models::ExecutionTargetSelection, spaces::assets::transcription::MediaTranscriptionProgress,
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
    pub vision_target: Option<ExecutionTargetSelection>,
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
            // A job cancelled mid-run keeps its cancelled row; the node data the execution already
            // persisted is returned so the caller does not fall behind the database.
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
            let target = crate::ai::settings::resolve(
                app,
                selected,
                crate::ai::contracts::ExecutionCapability::VisionUnderstanding,
            )
            .map_err(|error| error.to_string())?;
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
        EnrichmentJobType::AnalyzeImage | EnrichmentJobType::AnalyzeVideoFirstFrame => {
            let target = request
                .vision_target
                .take()
                .ok_or_else(|| "Visual analysis requires an AI connection target".to_owned())?;
            let target = crate::ai::settings::resolve(
                app,
                target,
                crate::ai::contracts::ExecutionCapability::VisionUnderstanding,
            )
            .map_err(|error| error.to_string())?;
            visual::analyze(app, &request, &mut data, target, cancellation).await?;
        }
        EnrichmentJobType::TranscribeMedia => {
            node::transcribe(app, job_id, &request, &mut data, progress).await?;
        }
        EnrichmentJobType::EmbedText | EnrichmentJobType::EmbedMediaTranscript => {
            return Err("Embedding jobs are owned by the retrieval index".to_owned());
        }
    }
    let data = node::save(app, &request, data).await?;
    Ok(EnrichmentJobOutput {
        node_id: request.node_id,
        data,
    })
}

async fn lane(job_type: EnrichmentJobType) -> Result<OwnedSemaphorePermit, String> {
    static MEDIA: OnceLock<Arc<Semaphore>> = OnceLock::new();
    static VISUAL: OnceLock<Arc<Semaphore>> = OnceLock::new();
    static DOCUMENT: OnceLock<Arc<Semaphore>> = OnceLock::new();
    let semaphore = match job_type {
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

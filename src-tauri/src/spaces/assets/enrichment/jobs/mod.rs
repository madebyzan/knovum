mod execution;
pub(crate) mod intent;
mod job;
mod persistence;

#[cfg(test)]
mod tests;

pub use execution::{EnrichmentJobOutput, RunEnrichmentJobRequest};
pub use job::{EnqueueEnrichmentJobRequest, EnrichmentJobRecord};

use tauri::{ipc::Channel, AppHandle, Runtime};

use crate::spaces::db;
use job::{FROM_LIVE, FROM_RUNNING, STATUS_CANCELLED, STATUS_FAILED, STATUS_SUCCEEDED};

#[tauri::command]
pub async fn run_enrichment_job(
    app: AppHandle,
    request: RunEnrichmentJobRequest,
    on_progress: Channel<crate::spaces::assets::transcription::MediaTranscriptionProgress>,
) -> Result<EnrichmentJobOutput, String> {
    execution::execute_job(app, request, on_progress).await
}

#[tauri::command]
pub async fn cancel_enrichment_job<R: Runtime>(
    app: AppHandle<R>,
    job_id: String,
) -> Result<(), String> {
    // The row is cancelled before the token is signalled so the execution being stopped cannot race
    // its own failure write in first. Cancelling an already finished job is a normal no-op.
    set_status(&app, &job_id, FROM_LIVE, STATUS_CANCELLED, None, false).await?;
    crate::ai::executions::cancel(&job_id)?;
    crate::spaces::assets::transcription::cancel_media_transcription(job_id).await
}

#[tauri::command]
pub async fn list_enrichment_jobs<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<Vec<EnrichmentJobRecord>, String> {
    let pool = db::pool(&app).await?;
    persistence::list_actionable(pool, &space_id).await
}

#[tauri::command]
pub async fn defer_enrichment_for_configuration<R: Runtime>(
    app: AppHandle<R>,
    job_id: String,
    error: String,
) -> Result<(), String> {
    if job_id.trim().is_empty() || error.trim().is_empty() {
        return Err("Configuration deferral requires a job id and reason".to_owned());
    }
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    if !persistence::defer_for_configuration(pool, &job_id, error, now).await? {
        return Err("Enrichment job is no longer actionable".to_owned());
    }
    Ok(())
}

pub(crate) async fn reconcile_interrupted(app: &tauri::AppHandle) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    persistence::reconcile_interrupted(pool, now).await
}

async fn set_status<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    from: &'static str,
    status: &str,
    error: Option<String>,
    increment_attempts: bool,
) -> Result<bool, String> {
    if job_id.trim().is_empty() {
        return Err("Enrichment job id is required".to_owned());
    }
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    persistence::set_status(pool, job_id, from, status, error, now, increment_attempts).await
}

fn validate_request(request: &EnqueueEnrichmentJobRequest) -> Result<(), String> {
    if request.space_id.trim().is_empty()
        || request.node_id.trim().is_empty()
        || request.source_revision.trim().is_empty()
    {
        return Err("Enrichment job requires space, node and source revision".to_owned());
    }
    Ok(())
}

async fn claim<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    request: &EnqueueEnrichmentJobRequest,
) -> Result<bool, String> {
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    persistence::claim_job(pool, job_id, request, now).await
}

async fn mark_succeeded<R: Runtime>(app: &AppHandle<R>, job_id: &str) -> Result<bool, String> {
    set_status(app, job_id, FROM_RUNNING, STATUS_SUCCEEDED, None, false).await
}

async fn mark_failed<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    error: String,
) -> Result<bool, String> {
    set_status(app, job_id, FROM_RUNNING, STATUS_FAILED, Some(error), false).await
}

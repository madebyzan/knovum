mod desktop_tasks;
mod execution;
pub(crate) use desktop_tasks::snapshot as desktop_tasks;
pub(crate) mod intent;
pub(crate) mod job;
mod persistence;
pub(crate) mod requests;
pub(crate) mod revision;
pub(crate) mod scheduler;

#[cfg(test)]
mod tests;

pub use job::{EnqueueEnrichmentJobRequest, EnrichmentJobProjection, EnrichmentJobState};

use tauri::{AppHandle, Runtime};

use crate::foundation::app_error::AppError;
use crate::spaces::db;
use job::{FROM_LIVE, FROM_RUNNING, STATUS_CANCELLED, STATUS_FAILED, STATUS_SUCCEEDED};

#[tauri::command]
pub async fn cancel_enrichment_job<R: Runtime>(
    app: AppHandle<R>,
    job_id: String,
) -> Result<(), AppError> {
    // The row is cancelled before the token is signalled so the execution being stopped cannot race
    // its own failure write in first. Cancelling an already finished job is a normal no-op.
    set_status(&app, &job_id, FROM_LIVE, STATUS_CANCELLED, None).await?;
    crate::ai::executions::cancel(&job_id)?;
    crate::spaces::assets::transcription::cancel_transcription(&job_id)?;
    Ok(())
}

#[tauri::command]
pub async fn list_actionable_enrichment_jobs<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Vec<EnrichmentJobProjection>, AppError> {
    let pool = db::pool(&app).await?;
    Ok((persistence::list_actionable_projection(pool, None).await)?)
}

pub(crate) async fn reconcile_interrupted(app: &tauri::AppHandle) -> Result<(), String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    persistence::reconcile_interrupted(pool, now).await
}

async fn set_status<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    from: &'static str,
    status: &str,
    error: Option<String>,
) -> Result<bool, String> {
    if job_id.trim().is_empty() {
        return Err("Enrichment job id is required".to_owned());
    }
    let now = crate::foundation::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    persistence::set_status(pool, job_id, from, status, error, now).await
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
    let now = crate::foundation::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    persistence::claim_job(pool, job_id, request, now).await
}

async fn mark_succeeded<R: Runtime>(app: &AppHandle<R>, job_id: &str) -> Result<bool, String> {
    set_status(app, job_id, FROM_RUNNING, STATUS_SUCCEEDED, None).await
}

async fn mark_failed<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    error: String,
) -> Result<bool, String> {
    set_status(app, job_id, FROM_RUNNING, STATUS_FAILED, Some(error)).await
}

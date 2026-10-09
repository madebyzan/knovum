use std::sync::OnceLock;

use tokio_util::sync::CancellationToken;

use crate::job_registry::{JobLease, JobRegistry};

fn jobs() -> &'static JobRegistry {
    static JOBS: OnceLock<JobRegistry> = OnceLock::new();
    JOBS.get_or_init(|| JobRegistry::new("Speech"))
}

pub fn acquire(job_id: &str) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
    uuid::Uuid::parse_str(job_id).map_err(|_| "Speech job identity is invalid")?;
    jobs().acquire_cancellable(job_id)
}

pub fn cancel(job_id: &str) -> Result<bool, String> {
    uuid::Uuid::parse_str(job_id).map_err(|_| "Speech job identity is invalid")?;
    jobs().cancel(job_id)
}

pub(super) async fn cancel_project_run(
    pool: &sqlx::SqlitePool,
    job_id: &str,
) -> Result<bool, String> {
    let active: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_runs WHERE id=? AND completed_at IS NULL \
         AND json_extract(input_snapshot,'$.workflow')=?)",
    )
    .bind(job_id)
    .bind(super::run_start::WORKFLOW)
    .fetch_one(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    if active {
        crate::spaces::runs::request_cancellation(pool, job_id).await?;
        if let Err(error) = crate::spaces::runs::cancel_active_attempts(pool, job_id).await {
            log::warn!(
                "Speech Run cancellation was accepted with a pending local interruption: {error}"
            );
        }
    }
    Ok(active)
}

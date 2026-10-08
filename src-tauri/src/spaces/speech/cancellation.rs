use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use tokio_util::sync::CancellationToken;

struct ActiveJob {
    token: CancellationToken,
}

fn jobs() -> &'static Mutex<HashMap<String, ActiveJob>> {
    static JOBS: OnceLock<Mutex<HashMap<String, ActiveJob>>> = OnceLock::new();
    JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn acquire(job_id: &str) -> Result<JobLease, String> {
    uuid::Uuid::parse_str(job_id).map_err(|_| "Speech job identity is invalid")?;
    let token = CancellationToken::new();
    let mut active = jobs()
        .lock()
        .map_err(|_| "Speech job registry is unavailable")?;
    if active.contains_key(job_id) {
        return Err("Speech job is already active".to_owned());
    }
    active.insert(
        job_id.to_owned(),
        ActiveJob {
            token: token.clone(),
        },
    );
    Ok(JobLease {
        job_id: job_id.to_owned(),
        token,
    })
}

pub fn cancel(job_id: &str) -> Result<bool, String> {
    uuid::Uuid::parse_str(job_id).map_err(|_| "Speech job identity is invalid")?;
    let active = jobs()
        .lock()
        .map_err(|_| "Speech job registry is unavailable")?;
    let Some(job) = active.get(job_id) else {
        return Ok(false);
    };
    job.token.cancel();
    Ok(true)
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
    }
    Ok(active)
}

pub struct JobLease {
    job_id: String,
    token: CancellationToken,
}

impl JobLease {
    pub fn token(&self) -> &CancellationToken {
        &self.token
    }
}

impl Drop for JobLease {
    fn drop(&mut self) {
        if let Ok(mut active) = jobs().lock() {
            active.remove(&self.job_id);
        }
    }
}

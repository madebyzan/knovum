use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy)]
pub enum JobKind {
    Preview,
}

struct ActiveJob {
    token: CancellationToken,
}

fn jobs() -> &'static Mutex<HashMap<String, ActiveJob>> {
    static JOBS: OnceLock<Mutex<HashMap<String, ActiveJob>>> = OnceLock::new();
    JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn acquire(job_id: &str, _kind: JobKind) -> Result<JobLease, String> {
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

pub fn cancel(job_id: &str, _persisted: bool) -> Result<bool, String> {
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

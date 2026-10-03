use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JobKind {
    Preview,
    Project,
}

struct ActiveJob {
    token: CancellationToken,
    kind: JobKind,
    accepts_unpersisted_cancel: bool,
}

fn jobs() -> &'static Mutex<HashMap<String, ActiveJob>> {
    static JOBS: OnceLock<Mutex<HashMap<String, ActiveJob>>> = OnceLock::new();
    JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn acquire(job_id: &str, kind: JobKind) -> Result<JobLease, String> {
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
            kind,
            accepts_unpersisted_cancel: true,
        },
    );
    Ok(JobLease {
        job_id: job_id.to_owned(),
        token,
    })
}

pub fn cancel(job_id: &str, persisted: bool) -> Result<bool, String> {
    uuid::Uuid::parse_str(job_id).map_err(|_| "Speech job identity is invalid")?;
    let active = jobs()
        .lock()
        .map_err(|_| "Speech job registry is unavailable")?;
    let Some(job) = active.get(job_id) else {
        return Ok(persisted);
    };
    job.token.cancel();
    Ok(persisted || job.kind == JobKind::Preview || job.accepts_unpersisted_cancel)
}

pub fn require_persisted_cancel(job_id: &str) -> Result<(), String> {
    let mut active = jobs()
        .lock()
        .map_err(|_| "Speech job registry is unavailable")?;
    if let Some(job) = active.get_mut(job_id) {
        job.accepts_unpersisted_cancel = false;
    }
    Ok(())
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

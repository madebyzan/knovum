use crate::job_registry::{JobLease, JobRegistry};
use std::sync::OnceLock;
use tokio_util::sync::CancellationToken;

pub(super) const CANCELLED: &str = "json-canvas-import-cancelled: JSON Canvas import cancelled";

fn jobs() -> &'static JobRegistry {
    static JOBS: OnceLock<JobRegistry> = OnceLock::new();
    JOBS.get_or_init(|| JobRegistry::new("JSON Canvas import"))
}

pub(super) fn begin(id: &str) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
    jobs().acquire_cancellable(id)
}

pub(super) fn cancel(id: &str) -> Result<(), String> {
    jobs().cancel(id)
}

pub(super) fn check(token: &CancellationToken) -> Result<(), String> {
    if token.is_cancelled() {
        Err(CANCELLED.into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;

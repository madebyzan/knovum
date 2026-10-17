use std::sync::OnceLock;

use tokio_util::sync::CancellationToken;

use crate::foundation::job_registry::{JobLease, JobRegistry};

fn jobs() -> &'static JobRegistry {
    static JOBS: OnceLock<JobRegistry> = OnceLock::new();
    JOBS.get_or_init(|| JobRegistry::new("Resource export"))
}

/// Encoding a long recording takes as long as it takes, so every export is
/// addressable and can be stopped by the person who started it.
pub(super) fn begin(
    job_id: &str,
) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
    identity(job_id)?;
    jobs().acquire_cancellable(job_id)
}

pub(super) fn cancel(job_id: &str) -> Result<bool, String> {
    identity(job_id)?;
    jobs().cancel(job_id)
}

fn identity(job_id: &str) -> Result<(), String> {
    uuid::Uuid::parse_str(job_id)
        .map(|_| ())
        .map_err(|_| "Export job identity is invalid".into())
}

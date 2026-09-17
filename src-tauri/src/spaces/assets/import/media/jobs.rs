use crate::job_registry::{JobLease, JobRegistry};
use std::sync::OnceLock;
use tokio_util::sync::CancellationToken;

fn registry() -> &'static JobRegistry {
    static REGISTRY: OnceLock<JobRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| JobRegistry::new("Media import"))
}

pub(super) fn acquire(
    job_id: String,
) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
    registry().acquire_cancellable(job_id)
}
pub(super) fn cancel(job_id: &str) -> Result<(), String> {
    registry().cancel(job_id)
}

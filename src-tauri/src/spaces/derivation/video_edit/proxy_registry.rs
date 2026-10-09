use std::sync::OnceLock;

use tokio_util::sync::CancellationToken;

use crate::job_registry::{JobLease, JobRegistry};

fn registry() -> &'static JobRegistry {
    static REGISTRY: OnceLock<JobRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| JobRegistry::new("Video edit preparation"))
}

pub fn acquire(job_id: String) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
    registry().acquire_cancellable(job_id)
}

pub fn cancel(job_id: &str) -> Result<(), String> {
    registry().cancel(job_id).map(|_| ())
}

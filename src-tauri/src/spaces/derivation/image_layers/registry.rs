use std::sync::OnceLock;

use tokio_util::sync::CancellationToken;

use crate::job_registry::{JobLease, JobRegistry};

fn registry() -> &'static JobRegistry {
    static REGISTRY: OnceLock<JobRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| JobRegistry::new("Image layer split"))
}

pub fn acquire(id: &str) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
    registry().acquire_cancellable(id)
}

pub fn cancel(id: &str) -> Result<(), String> {
    registry().cancel(id)
}

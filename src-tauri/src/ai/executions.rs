use std::sync::OnceLock;

use tokio_util::sync::CancellationToken;

use crate::job_registry::{JobLease, JobRegistry};

fn registry() -> &'static JobRegistry {
    static REGISTRY: OnceLock<JobRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| JobRegistry::new("AI execution"))
}

pub fn acquire(
    execution_id: impl Into<String>,
) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
    registry().acquire_cancellable(execution_id)
}

pub fn cancel(execution_id: &str) -> Result<(), String> {
    registry().cancel(execution_id)
}

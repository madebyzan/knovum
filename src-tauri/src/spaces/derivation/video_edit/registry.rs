use std::sync::OnceLock;

use tokio_util::sync::CancellationToken;

use crate::foundation::job_registry::{JobLease, JobRegistry};

fn registry() -> &'static JobRegistry {
    static REGISTRY: OnceLock<JobRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| JobRegistry::new("Video edit render"))
}

pub fn acquire(
    derivation_id: String,
) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
    registry().acquire_cancellable(derivation_id)
}

pub fn cancel(derivation_id: &str) -> Result<(), String> {
    registry().cancel(derivation_id).map(|_| ())
}

use crate::foundation::job_registry::{JobLease, JobRegistry};
use tokio_util::sync::CancellationToken;

/// Indexing a Space can run for minutes, so it is a job like any other long Rust work: it has an
/// identity the Task Center can show, and a token the person can pull.
///
/// Cancellation is only ever observed between batches in `generation_build::run` — never inside a
/// vector write and never between a completed build and `generation::activate`. A cancelled pass
/// therefore leaves committed markers for the chunks it did encode and the previous generation
/// still answering searches; the next pass resumes from exactly there.
fn indexings() -> &'static JobRegistry {
    static INDEXINGS: std::sync::OnceLock<JobRegistry> = std::sync::OnceLock::new();
    INDEXINGS.get_or_init(|| JobRegistry::new("Knowledge indexing"))
}

pub(crate) fn acquire(
    job_id: String,
) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
    indexings().acquire_cancellable(job_id)
}

pub(crate) fn cancel(job_id: &str) -> Result<bool, String> {
    indexings().cancel(job_id)
}

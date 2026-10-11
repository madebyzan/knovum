use crate::foundation::job_registry::{JobLease, JobRegistry};
use tokio_util::sync::CancellationToken;

fn searches() -> &'static JobRegistry {
    static SEARCHES: std::sync::OnceLock<JobRegistry> = std::sync::OnceLock::new();
    SEARCHES.get_or_init(|| JobRegistry::new("Knowledge search"))
}

pub(crate) fn acquire(
    job_id: String,
) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
    searches().acquire_cancellable(job_id)
}

pub(crate) fn cancel(job_id: &str) -> Result<(), String> {
    searches().cancel(job_id).map(|_| ())
}

#[cfg(test)]
mod tests;

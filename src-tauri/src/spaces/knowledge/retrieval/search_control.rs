use crate::job_registry::{JobLease, JobRegistry};
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
    searches().cancel(job_id)
}

#[cfg(test)]
mod tests {
    #[test]
    fn cancellation_reaches_the_active_search_and_lease_release_allows_retry() {
        let job_id = uuid::Uuid::new_v4().to_string();
        let (token, lease) = super::acquire(job_id.clone()).expect("search");
        assert!(super::acquire(job_id.clone()).is_err());
        super::cancel(&job_id).expect("cancel");
        assert!(token.is_cancelled());
        drop(lease);
        super::cancel(&job_id).expect("late cancel is harmless");
        let (retry_token, _lease) = super::acquire(job_id).expect("retry");
        assert!(!retry_token.is_cancelled());
    }
}

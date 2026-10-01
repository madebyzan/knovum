
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

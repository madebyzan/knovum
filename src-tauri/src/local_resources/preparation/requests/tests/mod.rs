use super::*;

#[test]
fn request_identity_and_title_are_enforced_at_the_registry_boundary() {
    for (job_id, title, expected) in [
        (" ", "Preparing resources", "job identifier"),
        ("resource-job", "\n", "task title"),
    ] {
        let error = begin(job_id, title, ResourcePreparationTarget::MediaImportTools)
            .err()
            .expect("invalid request must be rejected");
        assert!(error.contains(expected), "unexpected error: {error}");
    }
}

#[test]
fn accepted_request_can_be_cancelled_before_runner_starts() {
    let job_id = format!("resource-acceptance:{}", uuid::Uuid::new_v4());
    let request = begin(
        &job_id,
        "Preparing media import",
        ResourcePreparationTarget::MediaImportTools,
    )
    .unwrap();

    assert!(reserve_maintenance().is_err());

    cancel(&job_id).unwrap();

    assert!(request.cancellation().is_cancelled());
    let error = RESOURCE_PREPARATION_CANCELLED.to_string();
    request.finish(Some(&error)).unwrap();
    let snapshot = list()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.job_id == job_id)
        .unwrap();
    assert!(matches!(
        snapshot.state,
        ResourcePreparationState::Cancelled
    ));
}

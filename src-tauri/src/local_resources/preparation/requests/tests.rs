use super::*;

#[test]
fn accepted_request_can_be_cancelled_before_runner_starts() {
    let job_id = format!("resource-acceptance:{}", uuid::Uuid::new_v4());
    let request = begin(
        &job_id,
        "Preparing media import",
        ResourcePreparationTarget::MediaImportTools,
    )
    .unwrap();

    cancel(&job_id).unwrap();

    assert!(request.cancellation().is_cancelled());
    let error = RESOURCE_PREPARATION_CANCELLED.to_string();
    request.finish(Some(&error));
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

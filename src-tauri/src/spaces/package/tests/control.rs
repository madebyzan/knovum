use super::*;

pub(crate) fn begin_test_job(operation_id: &str) -> Result<JobLease<CancellationToken>, String> {
    begin(operation_id)
        .map(|(_, lease)| lease)
        .map_err(|error| error.message)
}

#[test]
fn operation_ids_require_canonical_bounded_uuids() {
    for invalid in [
        "",
        "00000000-0000-4000-8000-00000000000",
        "00000000-0000-4000-8000-0000000000000",
        "00000000-0000-4000-8000-00000000000A",
        "{00000000-0000-4000-8000-000000000000}",
    ] {
        let begin_error = begin(invalid).err().expect("invalid begin ID");
        let cancel_error = cancel(invalid).expect_err("invalid cancel ID");
        assert_eq!(begin_error.code, "invalid_request");
        assert_eq!(cancel_error.code, "invalid_request");
    }
    validate_operation_id("00000000-0000-4000-8000-000000000000").expect("canonical UUID");
}

#[test]
fn cleanup_context_preserves_typed_cancellation() {
    let error = with_cleanup_error(
        AppError::cancelled(CANCELLED),
        "package staging cleanup failed",
        "access denied",
    );
    assert_eq!(error.code, "cancelled");
    assert!(error.message.contains("access denied"));
}

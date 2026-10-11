use super::super::cancellation::ActiveRequest;

#[test]
fn stale_attempt_cannot_cancel_a_reused_request_identifier() {
    let active = ActiveRequest::default();
    let current = active.admit("job", 2, u64::MAX).unwrap();
    assert!(!active.cancel("job", 1).unwrap());
    current.check().unwrap();
    assert!(active.admit("other", 1, u64::MAX).is_err());
    assert!(active.cancel("job", 2).unwrap());
    assert_eq!(current.check().unwrap_err(), "cancelled");
    active.finish().unwrap();
    active.admit("next", 1, u64::MAX).unwrap().check().unwrap();
}

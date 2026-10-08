use super::parse_status;

#[test]
fn recognizes_documented_waiting_states_and_rejects_unknown_status() {
    for status in ["Preparing", "Queueing"] {
        assert!(parse_status(status).is_ok());
    }
    let error = parse_status("WaitingForever").unwrap_err();
    assert_eq!(error.code, "invalid_response");
}

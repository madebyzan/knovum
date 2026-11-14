use super::*;

#[test]
fn node_adapter_fails_when_npx_is_missing() {
    let err = node_adapter("any-package", "").expect_err("should fail when npx not found");
    assert_eq!(err.code, "agent_unavailable");
    assert!(err
        .message
        .contains("Node.js package runner (npx) was not found in PATH"));
}

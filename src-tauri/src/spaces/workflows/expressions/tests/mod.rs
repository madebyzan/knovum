use super::validate;

#[test]
fn rejects_scripts_and_accepts_bounded_boolean_expressions() {
    assert!(validate("input.kind == 'video' && input.durationMs > 0").is_ok());
    assert!(validate("while (true) {}").is_err());
}

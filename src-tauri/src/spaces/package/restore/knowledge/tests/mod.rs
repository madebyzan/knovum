use super::validate_relation_lifecycle;

#[test]
fn invalidated_relation_requires_prior_state() {
    assert!(validate_relation_lifecycle("invalidated", Some("active")).is_ok());
    assert!(validate_relation_lifecycle("invalidated", Some("contested")).is_ok());
    assert!(validate_relation_lifecycle("invalidated", None).is_err());
    assert!(validate_relation_lifecycle("invalidated", Some("invalidated")).is_err());
}

#[test]
fn current_relation_must_not_carry_prior_state() {
    assert!(validate_relation_lifecycle("active", None).is_ok());
    assert!(validate_relation_lifecycle("contested", None).is_ok());
    assert!(validate_relation_lifecycle("active", Some("contested")).is_err());
}

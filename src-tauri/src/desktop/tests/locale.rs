use super::*;

#[test]
fn unresolved_system_language_is_rejected() {
    assert!(labels(LocalePreference::System).is_err());
}

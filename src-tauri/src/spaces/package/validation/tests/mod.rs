use super::validate_manifest_version;

#[test]
fn accepts_only_current_space_packages() {
    assert!(validate_manifest_version(r#"{"format":"knovum-space","version":1}"#).is_ok());
    assert!(validate_manifest_version(r#"{"format":"knovum-space","version":2}"#).is_err());
    assert!(validate_manifest_version(r#"{"format":"other","version":1}"#).is_err());
}

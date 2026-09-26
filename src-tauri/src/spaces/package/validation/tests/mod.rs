use super::validate_manifest_version;

#[test]
fn accepts_only_current_space_packages() {
    let version = crate::spaces::package::PACKAGE_VERSION;
    assert!(validate_manifest_version(&format!(
        r#"{{"format":"knovum-space","version":{version}}}"#
    ))
    .is_ok());
    for obsolete in [1, 2, version + 1] {
        assert!(validate_manifest_version(&format!(
            r#"{{"format":"knovum-space","version":{obsolete}}}"#
        ))
        .is_err());
    }
    assert!(
        validate_manifest_version(&format!(r#"{{"format":"other","version":{version}}}"#)).is_err()
    );
}

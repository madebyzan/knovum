use super::{catalog, platform_directory};

#[test]
fn unsupported_media_tools_are_a_status_but_preparation_still_requires_a_descriptor() {
    let supported = matches!(platform_directory(), "macos-arm64" | "windows-x64");
    for kind in [
        catalog::MediaImportToolKind::YtDlp,
        catalog::MediaImportToolKind::Deno,
    ] {
        assert_eq!(catalog::find_descriptor(kind).unwrap().is_some(), supported);
        let required = catalog::descriptor(kind);
        assert_eq!(required.is_ok(), supported);
        if !supported {
            assert_eq!(
                required.err().unwrap(),
                "Media import tools are unavailable for this platform"
            );
        }
    }
}


use super::*;
#[test]
fn rejects_absolute_and_traversing_resource_references() {
    for reference in [
        "/tmp/image.png",
        "../image.png",
        "assets/../../image.png",
        "",
    ] {
        assert!(resolve_resource(Path::new("/tmp/canvas"), reference)
            .unwrap_err()
            .contains("source directory"));
    }
}

use std::path::Path;

use super::clipboard_contains_source;

#[test]
fn authorizes_only_the_exact_clipboard_resource() {
    let paths = vec!["/tmp/knowledge-source.pdf".to_owned()];
    assert!(clipboard_contains_source(
        Path::new("/tmp/knowledge-source.pdf"),
        &paths
    ));
    assert!(!clipboard_contains_source(
        Path::new("/tmp/other.pdf"),
        &paths
    ));
    let windows_path = r"C:\Knowledge\report.pdf";
    assert!(clipboard_contains_source(
        Path::new(windows_path),
        &[windows_path.to_owned()]
    ));
}

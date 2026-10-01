
use super::*;

#[test]
fn markdown_import_preserves_heading_and_filename_behavior() {
    assert_eq!(
        markdown_document("NOTE.MD", "# Research ##\r\n\r\nBody"),
        ("Research".into(), "Body".into())
    );
    assert_eq!(
        markdown_document("NOTE.MARKDOWN", "## Keep heading\nBody"),
        ("NOTE".into(), "## Keep heading\nBody".into())
    );
    assert_eq!(
        markdown_document("note.md", "# C#\nBody"),
        ("C#".into(), "Body".into())
    );
}

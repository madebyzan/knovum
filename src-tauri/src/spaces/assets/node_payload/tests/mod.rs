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

fn markdown(source_revision: Option<&str>) -> PreparedAsset {
    PreparedAsset {
        name: "PPT摘要.md".into(),
        path: None,
        kind: AssetKind::Markdown,
        text: Some("# 摘要\nBody".into()),
        preview_text: None,
        preview_language: None,
        preview_path: None,
        extracted_text: None,
        size: None,
        modified_at: None,
        source_revision: source_revision.map(str::to_owned),
        processing_state: None,
        intrinsic_width: None,
        intrinsic_height: None,
        knowledge_chunks: None,
        document_coverage: None,
        entries: None,
    }
}

#[test]
fn an_imported_markdown_node_omits_a_revision_it_does_not_have() {
    // A Markdown file published by a desktop Agent carried `"sourceRevision": null`, which the
    // node schema rejects, so the publication failed.
    let data = from_import(markdown(None)).unwrap();
    assert_eq!(
        data,
        serde_json::json!({"kind":"markdown", "title":"摘要", "text":"Body"})
    );
}

#[test]
fn an_imported_markdown_node_keeps_the_revision_it_has() {
    let data = from_import(markdown(Some("sha256:abc"))).unwrap();
    assert_eq!(data["sourceRevision"], "sha256:abc");
}

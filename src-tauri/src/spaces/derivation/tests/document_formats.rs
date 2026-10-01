use crate::documents::generation::{document::DocumentSource, pdf};

#[test]
fn typst_source_preserves_headings_lists_and_unicode() {
    let source = pdf::tests::render_source(&DocumentSource {
        markdown: "# 标题\n\n- 第一项\n- **Second** item".into(),
        ..Default::default()
    })
    .expect("generate Typst source");
    assert!(source.contains("#heading(level: 1)"));
    assert!(source.contains("第一项"));
    assert!(source.contains("#strong[#text(\"Second\")]"));
}

#[test]
fn renders_a_valid_pdf_with_platform_cjk_font() {
    let bytes = pdf::render(&DocumentSource {
        title: "知识文档".into(),
        markdown: "Hello, 世界。".into(),
        ..Default::default()
    })
    .expect("render PDF");
    assert!(bytes.starts_with(b"%PDF-"));
}

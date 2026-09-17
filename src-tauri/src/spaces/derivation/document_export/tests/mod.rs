use crate::documents::generation::pdf;

#[test]
fn typst_source_preserves_headings_lists_and_unicode() {
    let source = pdf::source_for_test("# 标题\n\n- 第一项\n- **Second** item");
    assert!(source.contains("= 标题"));
    assert!(source.contains("- 第一项"));
    assert!(source.contains("*Second* item"));
}

#[test]
fn renders_a_valid_pdf_with_bundled_cjk_font() {
    let bytes = pdf::render("# 知识文档\n\nHello, 世界。").expect("render PDF");
    assert!(bytes.starts_with(b"%PDF-"));
}

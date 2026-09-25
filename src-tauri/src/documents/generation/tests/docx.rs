use super::super::{docx, markdown};
use super::{fixture, source};

#[test]
fn docx_contains_real_tables_numbering_styles_and_hyperlinks() {
    let document = markdown::parse(&fixture()).unwrap();
    let package = docx::build_document(&document).unwrap().build();
    let body = String::from_utf8(package.document).unwrap();
    let styles = String::from_utf8(package.styles).unwrap();
    let numbering = String::from_utf8(package.numberings).unwrap();
    let relationships = String::from_utf8(package.document_rels).unwrap();
    for marker in ["<w:tbl>", "<w:numPr>", "<w:hyperlink", "KnovumCode", "中文 English document"] {
        assert!(body.contains(marker), "missing {marker}");
    }
    assert!(numbering.contains("w:val=\"7\""));
    assert!(numbering.contains("decimal"));
    assert!(numbering.contains("bullet"));
    assert!(relationships.contains("https://example.com/reference"));
    assert_eq!(styles.matches("w:styleId=\"Normal\"").count(), 1);
    assert!(styles.contains("Heading6"));
}

#[test]
fn code_and_citation_control_syntax_stays_escaped_in_xml() {
    let document = markdown::parse(&source("`<>&`\n\n[Link](https://example.com/?a=1&b=2)")).unwrap();
    let package = docx::build_document(&document).unwrap().build();
    let body = String::from_utf8(package.document).unwrap();
    let relationships = String::from_utf8(package.document_rels).unwrap();
    assert!(body.contains("&lt;&gt;&amp;"));
    assert!(relationships.contains("a=1&amp;b=2"));
}

#[test]
fn docx_is_packaged_as_a_zip_document() {
    assert!(docx::render(&fixture()).unwrap().starts_with(b"PK"));
}

#[test]
fn binary_export_rejects_unimplemented_images_and_unsafe_links() {
    for markdown in ["![Diagram](image.png)", "[Link](javascript:alert)", "[Link](file:///private/a)"] {
        assert!(docx::render(&source(markdown)).is_err());
    }
}

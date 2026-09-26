use std::io::{Cursor, Read};

#[test]
fn presentation_archive_preserves_text_and_escapes_xml() {
    let bytes = super::super::pptx::render(&super::super::document::DocumentSource {
        title: "Title & <evidence>".into(),
        markdown: "# Section\n\nVerified content".into(),
        citations: vec![],
    })
    .unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    for path in [
        "[Content_Types].xml",
        "ppt/presentation.xml",
        "ppt/slideMasters/slideMaster1.xml",
    ] {
        assert!(archive.by_name(path).is_ok(), "missing {path}");
    }
    let mut cover = String::new();
    archive
        .by_name("ppt/slides/slide1.xml")
        .unwrap()
        .read_to_string(&mut cover)
        .unwrap();
    assert!(cover.contains("Title &amp; &lt;evidence&gt;"));
    let mut content = String::new();
    archive
        .by_name("ppt/slides/slide2.xml")
        .unwrap()
        .read_to_string(&mut content)
        .unwrap();
    assert!(content.contains("Section"));
    assert!(content.contains("Verified content"));
    assert!(archive.by_name("ppt/slides/_rels/slide2.xml.rels").is_ok());
}

#[test]
fn oversized_presentation_fails_instead_of_silently_dropping_slides() {
    let markdown = (0..100)
        .map(|index| format!("# Slide {index}\n\nBody\n\n"))
        .collect::<String>();
    let error = super::super::pptx::render(&super::source(&markdown)).unwrap_err();
    assert!(error.starts_with("presentation-slide-limit:"));
}

#[test]
fn presentation_uses_shared_markdown_validation() {
    let error = super::super::pptx::render(&super::source("bad\u{0001}input")).unwrap_err();
    assert!(error.contains("control characters"));
}

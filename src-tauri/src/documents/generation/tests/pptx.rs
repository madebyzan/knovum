use std::io::Cursor;

use office_oxide::{Document, DocumentFormat};

use super::super::document::DocumentSource;

fn open_presentation(source: &DocumentSource) -> Document {
    let bytes = super::super::pptx::render(source).unwrap();
    Document::from_reader(Cursor::new(bytes), DocumentFormat::Pptx).unwrap()
}

#[test]
fn presentation_round_trips_semantic_text_and_metadata() {
    let source = DocumentSource {
        title: "Title & <证据> 🚀".into(),
        markdown: "# Section & <一>\n\nVerified 内容 & symbols".into(),
        citations: vec![],
    };

    let document = open_presentation(&source);
    let ir = document.to_ir();
    assert_eq!(ir.metadata.title.as_deref(), Some(source.title.as_str()));
    assert_eq!(ir.metadata.author.as_deref(), Some("Knovum"));
    assert_eq!(ir.sections.len(), 2);
    assert_eq!(ir.sections[0].title.as_deref(), Some(source.title.as_str()));
    assert_eq!(ir.sections[1].title.as_deref(), Some("Section & <一>"));

    let text = document.plain_text();
    assert!(text.contains("Title & <证据> 🚀"));
    assert!(text.contains("Section & <一>"));
    assert!(text.contains("Verified 内容 & symbols"));
}

#[test]
fn presentation_supports_exactly_one_hundred_slides() {
    let markdown = (0..99)
        .map(|index| format!("# Slide {index}\n\nBody\n\n"))
        .collect::<String>();

    let document = open_presentation(&super::source(&markdown));
    assert_eq!(document.to_ir().sections.len(), 100);
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
fn unsupported_control_characters_fail_shared_markdown_validation() {
    let error = super::super::pptx::render(&super::source("bad\u{0001}input")).unwrap_err();
    assert!(error.contains("control characters"));
}

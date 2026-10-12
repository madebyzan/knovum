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
        images: vec![],
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

    let mut source = super::source(&markdown);
    source.title = "Cover".into();
    let document = open_presentation(&source);
    assert_eq!(document.to_ir().sections.len(), 100);
}

#[test]
fn oversized_presentation_fails_instead_of_silently_dropping_slides() {
    let markdown = (0..100)
        .map(|index| format!("# Slide {index}\n\nBody\n\n"))
        .collect::<String>();
    let mut source = super::source(&markdown);
    source.title = "Cover".into();
    let error = super::super::pptx::render(&source).unwrap_err();
    assert!(error.starts_with("presentation-slide-limit:"));
}

#[test]
fn unsupported_control_characters_fail_shared_markdown_validation() {
    let error = super::super::pptx::render(&super::source("bad\u{0001}input")).unwrap_err();
    assert!(error.contains("control characters"));
}

#[test]
fn long_cjk_body_is_paginated_without_losing_text() {
    let body = "中文内容以及组合字符e\u{301}👨‍👩‍👧‍👦".repeat(150);
    let source = super::source(&format!("# 报告\n\n{body}"));
    let document = open_presentation(&source);
    let ir = document.to_ir();
    assert!(ir.sections.len() > 1);
    let joined = ir
        .sections
        .iter()
        .flat_map(|section| &section.elements)
        .filter_map(|element| match element {
            office_oxide::ir::Element::Paragraph(paragraph) => {
                Some(office_oxide::ir::inline_to_text(&paragraph.content))
            }
            _ => None,
        })
        .collect::<String>();
    assert_eq!(joined, body);
}

#[test]
fn presentation_keeps_rich_runs_real_tables_and_hyperlinks() {
    let bytes = super::super::pptx::render(&super::source(
        "# Section\n\n**Bold** and [Link](https://example.com)\n\n| A | B |\n|---|---|\n| C | D |",
    ))
    .unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let names = archive
        .file_names()
        .filter(|name| name.starts_with("ppt/slides/"))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut xml = String::new();
    for name in names {
        std::io::Read::read_to_string(&mut archive.by_name(&name).unwrap(), &mut xml).unwrap();
    }
    assert!(xml.contains("<a:tbl") && xml.contains("b=\"1\""));
    assert!(xml.contains("https://example.com"));
}

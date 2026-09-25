use super::super::{document::{Block, Citation, Inline}, markdown, text};
use super::{fixture, source};

#[test]
fn title_is_retained_without_duplicating_a_matching_leading_heading() {
    let mut input = source("# **Report**\n\nBody");
    input.title = "Report".into();
    let document = markdown::parse(&input).unwrap();
    assert_eq!(document.title, "Report");
    assert_eq!(document.blocks, vec![Block::Paragraph(vec![Inline::Text("Body".into())])]);
}

#[test]
fn a_distinct_body_heading_is_not_removed() {
    let mut input = source("# Introduction");
    input.title = "Report".into();
    let document = markdown::parse(&input).unwrap();
    assert!(matches!(document.blocks[0], Block::Heading { level: 1, .. }));
}

#[test]
fn citations_are_literal_structured_content() {
    let mut input = source("Body");
    input.citations.push(Citation {
        title: "[title] **not markup**".into(),
        quote: "*quote*\n# not a heading".into(),
    });
    let document = markdown::parse(&input).unwrap();
    let Block::Paragraph(content) = document.blocks.last().unwrap() else {
        panic!("expected a source paragraph");
    };
    assert_eq!(content.len(), 3);
    assert_eq!(text::inline_text(content, true), "[C1] [title] **not markup** — *quote*\n# not a heading");
}

#[test]
fn tight_lists_preserve_nested_block_structure() {
    let document = markdown::parse(&source("- outer\n  - inner\n- sibling")).unwrap();
    let Block::List { start: None, items } = &document.blocks[0] else {
        panic!("expected a bullet list");
    };
    assert_eq!(items.len(), 2);
    assert!(matches!(items[0][0], Block::Paragraph(_)));
    assert!(matches!(items[0][1], Block::List { .. }));
}

#[test]
fn common_fixture_preserves_tables_code_and_ordered_start() {
    let document = markdown::parse(&fixture()).unwrap();
    assert!(document.blocks.iter().any(|b| matches!(b, Block::Table(_))));
    assert!(document.blocks.iter().any(|b| matches!(b, Block::List { start: Some(7), .. })));
    assert!(document.blocks.iter().any(|b| matches!(b, Block::Code { language, .. } if language == "rust")));
}

#[test]
fn unsupported_html_is_not_silently_removed() {
    for markdown in ["<script>danger()</script>", "text <b>bold</b>"] {
        assert!(markdown::parse(&source(markdown)).unwrap_err().contains("HTML"));
    }
}

#[test]
fn input_size_and_nesting_are_bounded() {
    assert!(markdown::parse(&source(&"x".repeat(4 * 1024 * 1024 + 1))).is_err());
    assert!(markdown::parse(&source(&format!("{}text", "> ".repeat(100)))).is_err());
}

#[test]
fn invalid_xml_characters_are_rejected_before_rendering() {
    for value in ["hello\0", "hello\u{ffff}", "hello\u{fffe}"] {
        assert!(markdown::parse(&source(value)).is_err());
    }
}

use serde_json::json;

use super::source;
use crate::documents::generation::{input::DocumentInput, DocumentFormat};

#[test]
fn rich_note_serializes_all_targets_with_merged_table_and_marks() {
    let content = json!({"type":"doc","content":[
        {"type":"heading","attrs":{"level":1},"content":[{"type":"text","text":"富文本标题"}]},
        {"type":"paragraph","attrs":{"textAlign":"center"},"content":[
            {"type":"text","text":"Styled body","marks":[{"type":"bold"},{"type":"italic"},
                {"type":"underline"},{"type":"superscript"},
                {"type":"highlight","attrs":{"color":"#fff2cc"}}]}
        ]},
        {"type":"table","content":[
            {"type":"tableRow","content":[
                {"type":"tableHeader","attrs":{"colspan":2,"rowspan":1,"background":"#ffeeaa"},
                    "content":[{"type":"paragraph","content":[{"type":"text","text":"Header"}]}]}
            ]},
            {"type":"tableRow","content":[
                {"type":"tableCell","attrs":{"colspan":1,"rowspan":2},
                    "content":[{"type":"paragraph","content":[{"type":"text","text":"A"}]}]},
                {"type":"tableCell","attrs":{"colspan":1,"rowspan":1},
                    "content":[{"type":"paragraph","content":[{"type":"text","text":"B"}]}]}
            ]},
            {"type":"tableRow","content":[
                {"type":"tableCell","attrs":{"colspan":1,"rowspan":1},
                    "content":[{"type":"paragraph","content":[{"type":"text","text":"C"}]}]}
            ]}
        ]}
    ]});
    for (index, format) in [
        DocumentFormat::Docx,
        DocumentFormat::Pdf,
        DocumentFormat::Pptx,
        DocumentFormat::Html,
        DocumentFormat::Text,
    ]
    .into_iter()
    .enumerate()
    {
        let bytes = DocumentInput::Note(source(content.clone()))
            .render(format)
            .unwrap_or_else(|error| panic!("format {index}: {error}"));
        match format {
            DocumentFormat::Docx | DocumentFormat::Pptx => assert!(bytes.starts_with(b"PK")),
            DocumentFormat::Pdf => assert!(bytes.starts_with(b"%PDF-")),
            DocumentFormat::Html | DocumentFormat::Text => {
                let text = String::from_utf8(bytes).unwrap();
                assert!(text.contains("富文本标题") && text.contains("Styled body"));
                assert!(text.contains("Header") && text.contains('C'));
            }
        }
    }
}

#[test]
fn docx_quotes_are_flowing_paragraphs_without_fixed_text_boxes() {
    use std::io::{Cursor, Read};

    let text = "Long quoted content 世界。".repeat(200);
    let source = source(json!({"type":"doc","content":[
        {"type":"blockquote","content":[
            {"type":"paragraph","content":[{"type":"text","text":text}]},
            {"type":"blockquote","content":[
                {"type":"paragraph","content":[{"type":"text","text":"Nested quote"}]}
            ]}
        ]}
    ]}));
    let ir = super::super::parse(&source, DocumentFormat::Docx).unwrap();
    for (index, indent) in [360, 720].into_iter().enumerate() {
        let office_oxide::ir::Element::Paragraph(paragraph) = &ir.sections[0].elements[index]
        else {
            panic!("quote must participate in document flow");
        };
        assert_eq!(paragraph.indent_left_twips, Some(indent));
    }
    let bytes = DocumentInput::Note(source)
        .render(DocumentFormat::Docx)
        .unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    archive
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    assert!(xml.contains(&text) && xml.contains("Nested quote"));
    assert!(!xml.contains("txbxContent"));
}

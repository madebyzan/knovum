use office_oxide::ir::{Element, InlineContent, ParagraphAlignment, VerticalAlign};
use serde_json::json;

use super::source;
use crate::documents::generation::{input::note::parse, DocumentFormat};

#[test]
fn styled_note_keeps_title_marks_alignment_and_task_state() {
    let mut source = source(json!({"type":"doc","content":[
        {"type":"heading","attrs":{"level":1,"textAlign":null},"content":[{"type":"text","text":"Plan"}]},
        {"type":"paragraph","attrs":{"textAlign":"center"},"content":[
            {"type":"text","text":"Rich","marks":[
                {"type":"bold"},{"type":"italic"},{"type":"strike"},{"type":"underline"},
                {"type":"superscript"},{"type":"highlight","attrs":{"color":"#fff2cc"}},
                {"type":"textStyle","attrs":{"fontSize":"18pt","color":"#123456","fontFamily":"Aptos"}}
            ]},{"type":"hardBreak"},{"type":"text","text":"link","marks":[{"type":"link","attrs":{"href":"https://example.com"}}]}
        ]},
        {"type":"taskList","content":[{"type":"taskItem","attrs":{"checked":true},"content":[
            {"type":"paragraph","content":[{"type":"text","text":"Done"}]},
            {"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"Nested"}]}]}]}
        ]}]}
    ]}));
    source.title = " Plan ".into();
    let ir = parse(&source, DocumentFormat::Docx).unwrap();
    assert_eq!(ir.metadata.title.as_deref(), Some("Plan"));
    assert_eq!(ir.metadata.author.as_deref(), Some("Knovum"));
    assert_eq!(ir.sections[0].elements.len(), 3);
    let Element::Paragraph(paragraph) = &ir.sections[0].elements[1] else {
        panic!()
    };
    assert_eq!(paragraph.alignment, Some(ParagraphAlignment::Center));
    let InlineContent::Text(span) = &paragraph.content[0] else {
        panic!()
    };
    assert!(span.bold && span.italic && span.strikethrough);
    assert_eq!(span.font_size_half_pt, Some(36));
    assert_eq!(span.color, Some([0x12, 0x34, 0x56]));
    assert_eq!(span.highlight, Some([0xff, 0xf2, 0xcc]));
    assert_eq!(span.vertical_align, Some(VerticalAlign::Superscript));
    let Element::List(list) = &ir.sections[0].elements[2] else {
        panic!()
    };
    let Element::Paragraph(task) = &list.items[0].content[0] else {
        panic!()
    };
    assert_eq!(office_oxide::ir::inline_to_text(&task.content), "[x] Done");
    assert_eq!(list.items[0].nested.as_ref().unwrap().level, 1);
}

#[test]
fn merged_table_keeps_headers_spans_color_and_alignment() {
    let source = source(json!({"type":"doc","content":[{"type":"table","content":[
        {"type":"tableRow","content":[
            {"type":"tableHeader","attrs":{"colspan":2,"rowspan":1,"colwidth":[120,180],"background":"#ffeeaa","align":"center"},"content":[{"type":"paragraph","content":[{"type":"text","text":"Header"}]}]}
        ]},
        {"type":"tableRow","content":[
            {"type":"tableCell","attrs":{"colspan":1,"rowspan":2,"colwidth":[120],"backgroundColor":"#010203","align":"right"},"content":[{"type":"paragraph","content":[{"type":"text","text":"A"}]}]},
            {"type":"tableCell","attrs":{"colspan":1,"rowspan":1,"colwidth":[180],"align":"left"},"content":[{"type":"paragraph","content":[{"type":"text","text":"B"}]}]}
        ]},
        {"type":"tableRow","content":[{"type":"tableCell","attrs":{"colspan":1,"rowspan":1,"colwidth":[180],"align":"left"},"content":[{"type":"paragraph","content":[{"type":"text","text":"C"}]}]}]}
    ]}]}));
    let ir = parse(&source, DocumentFormat::Docx).unwrap();
    let Element::Table(table) = &ir.sections[0].elements[0] else {
        panic!()
    };
    assert!(table.rows[0].is_header);
    assert_eq!(table.rows[0].cells[0].col_span, 2);
    assert_eq!(
        table.rows[0].cells[0].background_color,
        Some([0xff, 0xee, 0xaa])
    );
    assert_eq!(table.rows[1].cells[0].row_span, 2);
    assert_eq!(
        table.rows[1].cells[0].text_align,
        Some(ParagraphAlignment::Right)
    );
    assert_eq!(table.column_widths_twips, vec![1800, 2700]);
}

#[test]
fn structural_blocks_and_literal_citations_remain_typed_ir() {
    let mut source = source(json!({"type":"doc","content":[
        {"type":"heading","attrs":{"level":6},"content":[{"type":"text","text":"Detail"}]},
        {"type":"orderedList","attrs":{"start":7,"type":null},"content":[
            {"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"Seventh","marks":[{"type":"code"},{"type":"subscript"}]}]}]}
        ]},
        {"type":"blockquote","content":[{"type":"paragraph","content":[{"type":"text","text":"Quoted"}]}]},
        {"type":"codeBlock","attrs":{"language":"rust"},"content":[{"type":"text","text":"let x = 1;"}]},
        {"type":"horizontalRule"}
    ]}));
    source
        .citations
        .push(crate::documents::generation::document::Citation {
            title: "**literal**".into(),
            quote: "<quoted>".into(),
        });
    let ir = parse(&source, DocumentFormat::Pdf).unwrap();
    let Element::List(list) = &ir.sections[0].elements[1] else {
        panic!()
    };
    assert!(list.ordered);
    assert_eq!(list.start_number, Some(7));
    let Element::TextBox(quote) = &ir.sections[0].elements[2] else {
        panic!()
    };
    let Element::Paragraph(quoted) = &quote.content[0] else {
        panic!()
    };
    assert_eq!(quoted.indent_left_twips, Some(360));
    let Element::CodeBlock(code) = &ir.sections[0].elements[3] else {
        panic!()
    };
    assert_eq!(code.language.as_deref(), Some("rust"));
    assert_eq!(code.content, "let x = 1;");
    assert!(matches!(
        &ir.sections[0].elements[4],
        Element::ThematicBreak
    ));
    let Element::Paragraph(citation) = &ir.sections[0].elements[6] else {
        panic!()
    };
    let InlineContent::Text(title) = &citation.content[1] else {
        panic!()
    };
    assert_eq!(title.text, "**literal**");
    assert!(title.bold);
}

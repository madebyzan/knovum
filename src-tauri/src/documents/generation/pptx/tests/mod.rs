use office_oxide::ir::{
    DocumentIR, Element, InlineContent, Paragraph, Section, Table, TableCell, TableRow, TextSpan,
};

#[test]
fn hard_breaks_are_typed_and_every_paragraph_chunk_fits() {
    let source = vec![InlineContent::Text(TextSpan::plain("line\n".repeat(50)))];
    let chunks = super::paragraphs::split(&source, 48, 9).unwrap();
    assert!(chunks.len() > 1);
    for chunk in chunks {
        assert!(chunk.lines <= 9);
        for inline in chunk.content {
            if let InlineContent::Text(span) = inline {
                assert!(!span.text.contains('\n'));
            }
        }
    }
}

#[test]
fn long_table_headers_do_not_reappear_as_unsplit_overflow() {
    let row = |text: String, header| TableRow {
        is_header: header,
        cells: vec![TableCell {
            col_span: 1,
            row_span: 1,
            content: vec![Element::Paragraph(Paragraph {
                content: vec![InlineContent::Text(TextSpan::plain(text))],
                ..Default::default()
            })],
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut rows = vec![row("表头".repeat(150), true)];
    rows.extend((0..20).map(|index| row(format!("Row {index}"), false)));
    let ir = DocumentIR {
        sections: vec![Section {
            elements: vec![Element::Table(Table {
                rows,
                ..Default::default()
            })],
            ..Default::default()
        }],
        ..Default::default()
    };
    let slides = super::pagination::paginate(&ir).unwrap();
    assert!(slides.sections.len() > 1);
    let mut header_text = String::new();
    for section in slides.sections {
        for element in &section.elements {
            if let Element::Table(table) = element {
                let mut height = 0;
                for row in &table.rows {
                    let mut lines = 0;
                    for cell in &row.cells {
                        for element in &cell.content {
                            if let Element::Paragraph(paragraph) = element {
                                let chunks =
                                    super::paragraphs::split(&paragraph.content, 48, 100).unwrap();
                                lines = lines.max(chunks[0].lines);
                                if row.is_header {
                                    header_text
                                        .push_str(&super::paragraphs::plain(&paragraph.content));
                                }
                            }
                        }
                    }
                    height += lines + 1;
                }
                assert!(height <= super::pagination::BODY_LINES);
            }
        }
    }
    assert_eq!(header_text, "表头".repeat(150));
}

#[test]
fn long_titles_keep_overflow_visible_and_preserve_all_text() {
    use unicode_width::UnicodeWidthStr;
    for title in ["长标题中文内容".repeat(100), "LongLatinHeading".repeat(100)] {
        let ir = DocumentIR {
            sections: vec![Section {
                elements: vec![Element::Heading(office_oxide::ir::Heading {
                    level: 1,
                    content: vec![InlineContent::Text(TextSpan::plain(&title))],
                    ..Default::default()
                })],
                ..Default::default()
            }],
            ..Default::default()
        };
        let slides = super::pagination::paginate(&ir).unwrap();
        let mut recovered = slides.sections[0].title.clone().unwrap();
        for section in &slides.sections {
            assert!(section.title.as_deref().unwrap().width_cjk() <= 32);
            for element in &section.elements {
                if let Element::Paragraph(paragraph) = element {
                    recovered.push_str(&super::paragraphs::plain(&paragraph.content));
                }
            }
        }
        assert_eq!(recovered, title);
        assert!(super::render_ir(&ir).unwrap().starts_with(b"PK"));
    }
}

#[test]
fn table_cells_fit_the_writers_fixed_row_height_and_width() {
    let cells = (0..3)
        .map(|_| TableCell {
            col_span: 1,
            row_span: 1,
            content: vec![Element::Paragraph(Paragraph {
                content: vec![InlineContent::Text(TextSpan::plain("长单元格W".repeat(20)))],
                ..Default::default()
            })],
            ..Default::default()
        })
        .collect::<Vec<_>>();
    let ir = DocumentIR {
        sections: vec![Section {
            elements: vec![Element::Table(Table {
                rows: vec![TableRow {
                    cells,
                    ..Default::default()
                }],
                ..Default::default()
            })],
            ..Default::default()
        }],
        ..Default::default()
    };
    for section in super::pagination::paginate(&ir).unwrap().sections {
        for element in &section.elements {
            let Element::Table(table) = element else {
                continue;
            };
            assert!(table.rows.len() <= 5);
            for cell in table.rows.iter().flat_map(|row| &row.cells) {
                let Element::Paragraph(paragraph) = &cell.content[0] else {
                    panic!()
                };
                let chunks = super::paragraphs::split(&paragraph.content, 40 / 3, 100).unwrap();
                assert_eq!(chunks[0].lines, 1);
            }
        }
    }
    let mut wide = ir.clone();
    let Element::Table(table) = &mut wide.sections[0].elements[0] else {
        panic!()
    };
    table.rows[0].cells.resize_with(13, TableCell::default);
    assert!(super::pagination::paginate(&wide)
        .unwrap_err()
        .contains("12-column"));
}

#[test]
fn numbering_outside_the_supported_range_fails_without_wrapping() {
    let ir = DocumentIR {
        sections: vec![Section {
            elements: vec![Element::List(office_oxide::ir::List {
                ordered: true,
                start_number: Some(u32::MAX),
                items: vec![
                    office_oxide::ir::ListItem {
                        content: vec![Element::Paragraph(Paragraph {
                            content: vec![InlineContent::Text(TextSpan::plain("First"))],
                            ..Default::default()
                        })],
                        nested: None,
                    },
                    office_oxide::ir::ListItem {
                        content: vec![Element::Paragraph(Paragraph {
                            content: vec![InlineContent::Text(TextSpan::plain("Second"))],
                            ..Default::default()
                        })],
                        nested: None,
                    },
                ],
                ..Default::default()
            })],
            ..Default::default()
        }],
        ..Default::default()
    };
    let error = super::render_ir(&ir).unwrap_err();
    assert!(error.contains("numbering range"), "{error}");
}

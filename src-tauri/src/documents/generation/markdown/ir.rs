use office_oxide::ir::{
    CodeBlock, DocumentIR, Element, Heading, Image, ImageFormat, List, ListItem, Metadata,
    Paragraph, ParagraphAlignment, Section, Table, TableCell, TableRow, TextBox,
};

use super::super::{
    document::{Alignment, Block, Document},
    images::Images,
};

pub(crate) fn translate(document: &Document, images: &Images) -> Result<DocumentIR, String> {
    Ok(DocumentIR {
        metadata: Metadata {
            title: Some(document.title.clone()),
            author: Some("Knovum".into()),
            ..Default::default()
        },
        sections: vec![Section {
            elements: blocks(&document.blocks, images)?,
            ..Default::default()
        }],
        ..Default::default()
    })
}

pub(super) fn blocks(content: &[Block], images: &Images) -> Result<Vec<Element>, String> {
    let mut result = Vec::new();
    for block in content {
        match block {
            Block::Paragraph(value) => result.extend(super::ir_inlines::paragraph(value, images)?),
            Block::Heading { level, content } => {
                result.push(Element::Heading(Heading {
                    level: *level,
                    content: super::ir_inlines::text(content)?,
                    ..Default::default()
                }));
                result.extend(super::ir_inlines::image_elements(content, images)?);
            }
            Block::Code { language, text } => result.push(Element::CodeBlock(CodeBlock {
                language: Some(language.clone()),
                content: text.clone(),
            })),
            Block::Quote(value) => result.push(Element::TextBox(TextBox {
                content: blocks(value, images)?,
                ..Default::default()
            })),
            Block::Rule => result.push(Element::ThematicBreak),
            Block::List { start, items } => result.push(Element::List(List {
                ordered: start.is_some(),
                start_number: start
                    .map(u32::try_from)
                    .transpose()
                    .map_err(|_| "Document list start exceeds numbering range")?,
                items: items
                    .iter()
                    .map(|item| {
                        Ok(ListItem {
                            content: blocks(item, images)?,
                            ..Default::default()
                        })
                    })
                    .collect::<Result<_, String>>()?,
                ..Default::default()
            })),
            Block::Table(value) => {
                let mut rows = Vec::new();
                for (index, row) in std::iter::once(&value.header)
                    .chain(&value.rows)
                    .enumerate()
                {
                    let mut cells = Vec::new();
                    for (content, alignment) in row.iter().zip(&value.alignments) {
                        let alignment = match alignment {
                            Alignment::Left => ParagraphAlignment::Left,
                            Alignment::Center => ParagraphAlignment::Center,
                            Alignment::Right => ParagraphAlignment::Right,
                        };
                        cells.push(TableCell {
                            content: super::ir_inlines::paragraph(content, images)?,
                            col_span: 1,
                            row_span: 1,
                            text_align: Some(alignment),
                            ..Default::default()
                        });
                    }
                    rows.push(TableRow {
                        cells,
                        is_header: index == 0,
                        height_twips: None,
                        height_rule: None,
                        allow_break: true,
                        repeat_as_header: index == 0,
                    });
                }
                result.push(Element::Table(Table {
                    rows,
                    ..Default::default()
                }));
            }
        }
    }
    Ok(result)
}

pub(super) fn image(destination: &str, alt: String, images: &Images) -> Result<Element, String> {
    let image = images.get(destination)?;
    Ok(Element::Image(Image {
        data: Some(images.clone_bytes(destination)?),
        format: Some(ImageFormat::Png),
        alt_text: Some(alt),
        pixel_width: Some(image.width),
        pixel_height: Some(image.height),
        ..Default::default()
    }))
}

pub(super) fn paragraph(content: Vec<office_oxide::ir::InlineContent>) -> Element {
    Element::Paragraph(Paragraph {
        content,
        ..Default::default()
    })
}

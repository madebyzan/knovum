mod blocks;
mod colors;
mod inlines;
mod lists;
mod media;
mod model;
mod preflight;
mod table_cells;
mod tables;

use office_oxide::{
    ir::{DocumentIR, Element, Heading, InlineContent, Metadata, Paragraph, Section, TextSpan},
    DocumentFormat as OfficeFormat,
};

use self::model::Node;
use super::super::{
    document::{Citation, NoteSource},
    images::Images,
    DocumentFormat,
};

pub(super) fn parse(source: &NoteSource, format: DocumentFormat) -> Result<DocumentIR, String> {
    preflight::validate(source)?;
    let root: Node = serde_json::from_value(source.content.clone())
        .map_err(|error| format!("Malformed Tiptap document: {error}"))?;
    validate_root(&root)?;
    let loaded_images = if matches!(format, DocumentFormat::Text) {
        None
    } else {
        Some(Images::from_sources(&source.images)?)
    };
    let context = Context {
        images: loaded_images.as_ref(),
        format,
    };
    let mut elements = blocks::parse(&root.content, &context, 1)?;
    let title = source.title.trim().to_owned();
    if !title.is_empty() && !has_leading_title(&elements, &title) {
        elements.insert(0, heading(1, title.clone()));
    }
    append_citations(&mut elements, &source.citations);
    Ok(DocumentIR {
        metadata: Metadata {
            format: OfficeFormat::Docx,
            title: (!title.is_empty()).then_some(title),
            author: Some("Knovum".into()),
            ..Default::default()
        },
        sections: vec![Section {
            elements,
            ..Default::default()
        }],
        defined_names: Vec::new(),
    })
}

pub(super) struct Context<'a> {
    images: Option<&'a Images>,
    format: DocumentFormat,
}

fn validate_root(root: &Node) -> Result<(), String> {
    if root.kind != "doc" {
        return Err("Tiptap document root must have type doc".into());
    }
    model::no_attrs(&root.attrs, "doc")?;
    if root.text.is_some() || !root.marks.is_empty() {
        return Err("Tiptap document root contains inline-only fields".into());
    }
    Ok(())
}

fn has_leading_title(elements: &[Element], title: &str) -> bool {
    elements.first().is_some_and(|element| {
        matches!(element, Element::Heading(heading)
            if heading.level == 1 && office_oxide::ir::inline_to_text(&heading.content).trim() == title)
    })
}

fn append_citations(elements: &mut Vec<Element>, citations: &[Citation]) {
    if citations.is_empty() {
        return;
    }
    elements.push(heading(2, "Sources".into()));
    for (index, citation) in citations.iter().enumerate() {
        elements.push(Element::Paragraph(Paragraph {
            content: vec![
                InlineContent::Text(TextSpan::plain(format!("[C{}] ", index + 1))),
                InlineContent::Text(TextSpan {
                    text: citation.title.clone(),
                    bold: true,
                    ..Default::default()
                }),
                InlineContent::Text(TextSpan::plain(format!(" — {}", citation.quote))),
            ],
            ..Default::default()
        }));
    }
}

fn heading(level: u8, text: String) -> Element {
    Element::Heading(Heading {
        level,
        content: vec![InlineContent::Text(TextSpan::plain(text))],
        ..Default::default()
    })
}

#[cfg(test)]
mod tests;

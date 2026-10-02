use office_oxide::{
    ir::{DocumentIR, Element},
    DocumentFormat,
};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::{
    budget, content,
    document_text::element_text,
    ensure_active,
    location::{is_word, location, update_heading_path},
    tables, DocumentSegment,
};

pub(super) fn extract(
    ir: &DocumentIR,
    format: DocumentFormat,
    cancellation: &CancellationToken,
    budget: &mut budget::TextBudget,
) -> Result<Vec<DocumentSegment>, String> {
    let mut segments = Vec::<DocumentSegment>::new();
    let mut headings = Vec::<String>::new();
    for (section_index, section) in ir.sections.iter().enumerate() {
        ensure_active(cancellation)?;
        if !is_word(format) {
            headings.clear();
        }
        for content in content::parts(section) {
            let mut part_headings = Vec::new();
            let headings = if content.part == "body" {
                &mut headings
            } else {
                &mut part_headings
            };
            for (element_index, element) in content.elements.iter().enumerate() {
                ensure_active(cancellation)?;
                if let Element::Table(table) = element {
                    let owner = location(
                        format,
                        section_index,
                        element_index,
                        section.title.as_deref(),
                        headings,
                        content.part,
                    );
                    segments.extend(tables::segments(table, owner, cancellation, budget)?);
                    continue;
                }
                let text = element_text(element)?;
                if text.trim().is_empty() {
                    continue;
                }
                budget.consume(&text)?;
                if let Element::Heading(heading) = element {
                    update_heading_path(headings, heading.level, text.trim());
                }
                let owner = location(
                    format,
                    section_index,
                    element_index,
                    section.title.as_deref(),
                    headings,
                    content.part,
                );
                if is_word(format) && matches!(element, Element::Paragraph(_)) {
                    if let Some(previous) = segments.last_mut() {
                        let contiguous = previous
                            .metadata
                            .get("elementEnd")
                            .and_then(serde_json::Value::as_u64)
                            == Some(element_index.saturating_sub(1) as u64);
                        if previous.kind == "text"
                            && previous.location["section"] == owner["section"]
                            && previous.location["headingPath"] == owner["headingPath"]
                            && previous.location["part"] == owner["part"]
                            && contiguous
                            && previous.text.encode_utf16().count() + text.encode_utf16().count()
                                < 4000
                        {
                            previous.text.push('\n');
                            previous.text.push_str(&text);
                            previous.metadata["elementEnd"] = json!(element_index);
                            continue;
                        }
                    }
                }
                segments.push(DocumentSegment {
                    kind: element_kind(format, element),
                    text,
                    location: owner,
                    metadata: json!({
                        "section": section_index,
                        "element": element_index,
                        "elementEnd": element_index,
                    }),
                });
            }
        }
    }
    Ok(segments)
}

fn element_kind(format: DocumentFormat, element: &Element) -> &'static str {
    if matches!(format, DocumentFormat::Ppt | DocumentFormat::Pptx) {
        "slide"
    } else if matches!(format, DocumentFormat::Xls | DocumentFormat::Xlsx) {
        if matches!(element, Element::Table(_)) {
            "table"
        } else {
            "sheet"
        }
    } else if matches!(element, Element::Heading(_)) {
        "heading"
    } else if matches!(element, Element::Table(_)) {
        "table"
    } else {
        "text"
    }
}

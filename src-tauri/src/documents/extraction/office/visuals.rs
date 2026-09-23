use super::{
    content,
    document_text::element_text,
    ensure_active, images,
    location::{is_word, location, update_heading_path},
};
use office_oxide::{
    ir::{DocumentIR, Element},
    DocumentFormat,
};
use tokio_util::sync::CancellationToken;

pub(super) fn collect(
    ir: &DocumentIR,
    format: DocumentFormat,
    cancellation: &CancellationToken,
) -> Result<images::DocumentImages, String> {
    let mut images = images::DocumentImages::default();
    let mut headings = Vec::new();
    for (section_index, section) in ir.sections.iter().enumerate() {
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
                if let Element::Heading(heading) = element {
                    update_heading_path(headings, heading.level, element_text(element)?.trim());
                }
                let location = location(
                    format,
                    section_index,
                    element_index,
                    section.title.as_deref(),
                    headings,
                    content.part,
                );
                let start = images.len();
                images::collect(
                    std::slice::from_ref(element),
                    &location,
                    "",
                    &format!("office:{section_index}/{}/{element_index}", content.part),
                    &mut images,
                    0,
                )?;
                if images.len() > start {
                    let mut context = String::new();
                    for neighbor in &content.elements[element_index.saturating_sub(1)
                        ..(element_index + 2).min(content.elements.len())]
                    {
                        let remaining = 4000 - context.encode_utf16().count();
                        if remaining == 0 {
                            break;
                        }
                        context.push_str(&super::super::text::bounded_utf16_text(
                            &element_text(neighbor)?,
                            remaining,
                        ));
                        if context.encode_utf16().count() < 4000 {
                            context.push('\n');
                        }
                    }
                    images.attach_context(start, context.trim());
                }
            }
        }
    }
    Ok(images)
}

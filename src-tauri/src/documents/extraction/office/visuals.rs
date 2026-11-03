use office_oxide::{
    ir::{DocumentIR, Element},
    DocumentFormat,
};
use tokio_util::sync::CancellationToken;

use super::{
    document_text::element_text,
    ensure_active, images,
    location::{is_word, location, update_heading_path},
};

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
        // Only collect figures from section body elements; Word page headers and footers carry
        // running logos, badges, and page watermarks rather than document figures.
        for (element_index, element) in section.elements.iter().enumerate() {
            ensure_active(cancellation)?;
            if let Element::Heading(heading) = element {
                update_heading_path(&mut headings, heading.level, element_text(element)?.trim());
            }
            let location = location(
                format,
                section_index,
                element_index,
                section.title.as_deref(),
                &headings,
                "body",
            );
            let start = images.len();
            images::collect(
                std::slice::from_ref(element),
                &location,
                "",
                &format!("office:{section_index}/body/{element_index}"),
                &mut images,
                0,
            )?;
            if images.len() > start {
                let mut context = String::new();
                for neighbor in &section.elements[element_index.saturating_sub(1)
                    ..(element_index + 2).min(section.elements.len())]
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
    Ok(images)
}

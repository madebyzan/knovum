use office_oxide::ir::{Element, InlineContent, TextSpan};

use super::super::{document::Inline, images::Images};

pub(super) fn text(content: &[Inline]) -> Result<Vec<InlineContent>, String> {
    let mut output = Vec::new();
    flatten(content, &TextSpan::default(), &mut output)?;
    Ok(output)
}

fn flatten(
    content: &[Inline],
    marks: &TextSpan,
    output: &mut Vec<InlineContent>,
) -> Result<(), String> {
    for inline in content {
        let mut span = marks.clone();
        match inline {
            Inline::Strong(children) => {
                span.bold = true;
                flatten(children, &span, output)?;
            }
            Inline::Emphasis(children) => {
                span.italic = true;
                flatten(children, &span, output)?;
            }
            Inline::Strike(children) => {
                span.strikethrough = true;
                flatten(children, &span, output)?;
            }
            Inline::Link { url, content } => {
                super::super::links::validate(url)?;
                span.hyperlink = Some(url.clone());
                flatten(content, &span, output)?;
            }
            Inline::HardBreak => output.push(InlineContent::LineBreak),
            inline => {
                span.text = match inline {
                    Inline::Text(value) => value.clone(),
                    Inline::SoftBreak => " ".into(),
                    Inline::Code(value) => {
                        span.font_name = Some("Courier New".into());
                        value.clone()
                    }
                    Inline::Image { alt, .. } => super::super::text::inline_text(alt, false),
                    Inline::Task(value) => {
                        if *value {
                            "[x] ".into()
                        } else {
                            "[ ] ".into()
                        }
                    }
                    _ => return Err("Unexpected Markdown inline content".into()),
                };
                output.push(InlineContent::Text(span));
            }
        }
    }
    Ok(())
}

pub(super) fn paragraph(content: &[Inline], images: &Images) -> Result<Vec<Element>, String> {
    let mut output = Vec::new();
    let mut pending = Vec::new();
    segments(
        content,
        &TextSpan::default(),
        images,
        &mut pending,
        &mut output,
    )?;
    if !pending.is_empty() {
        output.push(super::ir::paragraph(pending));
    }
    Ok(output)
}

fn segments(
    content: &[Inline],
    marks: &TextSpan,
    images: &Images,
    pending: &mut Vec<InlineContent>,
    output: &mut Vec<Element>,
) -> Result<(), String> {
    for inline in content {
        let mut span = marks.clone();
        match inline {
            Inline::Strong(children) => {
                span.bold = true;
                segments(children, &span, images, pending, output)?;
            }
            Inline::Emphasis(children) => {
                span.italic = true;
                segments(children, &span, images, pending, output)?;
            }
            Inline::Strike(children) => {
                span.strikethrough = true;
                segments(children, &span, images, pending, output)?;
            }
            Inline::Link { url, content } => {
                super::super::links::validate(url)?;
                span.hyperlink = Some(url.clone());
                segments(content, &span, images, pending, output)?;
            }
            Inline::Image { destination, alt } => {
                if !pending.is_empty() {
                    output.push(super::ir::paragraph(std::mem::take(pending)));
                }
                let mut element = super::ir::image(
                    destination,
                    super::super::text::inline_text(alt, false),
                    images,
                )?;
                if let Element::Image(image) = &mut element {
                    image.hyperlink = span.hyperlink;
                }
                output.push(element);
            }
            _ => flatten(std::slice::from_ref(inline), &span, pending)?,
        }
    }
    Ok(())
}

pub(super) fn image_elements(content: &[Inline], images: &Images) -> Result<Vec<Element>, String> {
    let mut result = Vec::new();
    for inline in content {
        match inline {
            Inline::Image { destination, alt } => result.push(super::ir::image(
                destination,
                super::super::text::inline_text(alt, false),
                images,
            )?),
            Inline::Strong(children)
            | Inline::Emphasis(children)
            | Inline::Strike(children)
            | Inline::Link {
                content: children, ..
            } => result.extend(image_elements(children, images)?),
            _ => {}
        }
    }
    Ok(result)
}

use office_oxide::ir::{
    CodeBlock, Element, Heading, InlineContent, Paragraph, ParagraphAlignment, TextBox, TextSpan,
};

use super::{inlines, lists, media, model, tables, Context};
use model::Node;

pub(super) fn parse(
    nodes: &[Node],
    context: &Context<'_>,
    depth: usize,
) -> Result<Vec<Element>, String> {
    if depth > 64 {
        return Err("Document export nesting exceeds 64 levels".into());
    }
    let mut elements = Vec::new();
    for node in nodes {
        if node.kind == "imageGroup" {
            reject_block_extras(node)?;
            elements.extend(image_group_content(node, context)?);
        } else if matches!(node.kind.as_str(), "blockquote" | "callout")
            && matches!(context.format, super::DocumentFormat::Docx)
        {
            reject_block_extras(node)?;
            let content = if node.kind == "callout" {
                callout_content(node, context, depth + 1)?
            } else {
                quote_content(node, context, depth + 1)?
            };
            elements.extend(content);
        } else {
            elements.push(block(node, context, depth)?);
        }
    }
    Ok(elements)
}

fn block(node: &Node, context: &Context<'_>, depth: usize) -> Result<Element, String> {
    reject_block_extras(node)?;
    match node.kind.as_str() {
        "paragraph" => paragraph(node),
        "heading" => heading(node),
        "bulletList" | "orderedList" | "taskList" => lists::parse(node, context, depth + 1, 0),
        "blockquote" => quote(node, context, depth + 1),
        "callout" => callout(node, context, depth + 1),
        "codeBlock" => code(node, context),
        "blockMath" => block_math(node),
        "horizontalRule" => empty_block(node, Element::ThematicBreak),
        "table" => tables::parse(node, context, depth + 1),
        "image" => media::image(node, context),
        kind => Err(format!("Unsupported Tiptap block node {kind}")),
    }
}

pub(super) fn paragraph(node: &Node) -> Result<Element, String> {
    model::allowed_attrs(&node.attrs, "paragraph", &["textAlign"])?;
    Ok(Element::Paragraph(Paragraph {
        content: inlines::parse(&node.content)?,
        alignment: alignment(model::string_attr(&node.attrs, "textAlign")?)?,
        ..Default::default()
    }))
}

fn heading(node: &Node) -> Result<Element, String> {
    model::allowed_attrs(&node.attrs, "heading", &["level", "textAlign"])?;
    let level = model::u32_attr(&node.attrs, "level", 1)?;
    if !(1..=6).contains(&level) {
        return Err("Tiptap heading levels must be between 1 and 6".into());
    }
    Ok(Element::Heading(Heading {
        level: level as u8,
        content: inlines::parse(&node.content)?,
        alignment: alignment(model::string_attr(&node.attrs, "textAlign")?)?,
        ..Default::default()
    }))
}

fn quote(node: &Node, context: &Context<'_>, depth: usize) -> Result<Element, String> {
    Ok(Element::TextBox(TextBox {
        content: quote_content(node, context, depth)?,
        ..Default::default()
    }))
}

fn callout(node: &Node, context: &Context<'_>, depth: usize) -> Result<Element, String> {
    Ok(Element::TextBox(TextBox {
        content: callout_content(node, context, depth)?,
        ..Default::default()
    }))
}

fn image_group_content(node: &Node, context: &Context<'_>) -> Result<Vec<Element>, String> {
    model::allowed_attrs(&node.attrs, "imageGroup", &["layout"])?;
    if let Some(layout) = model::string_attr(&node.attrs, "layout")? {
        if !matches!(
            layout,
            "auto" | "gallery" | "compare" | "masonry" | "cards" | "stack"
        ) {
            return Err(format!("Unsupported Tiptap imageGroup layout {layout}"));
        }
    }
    let mut elements = Vec::with_capacity(node.content.len());
    for child in &node.content {
        if child.kind != "image" {
            return Err(format!(
                "Unsupported child node {} inside Tiptap imageGroup",
                child.kind
            ));
        }
        reject_block_extras(child)?;
        elements.push(media::image(child, context)?);
    }
    Ok(elements)
}

fn callout_label(kind: Option<&str>) -> Result<&'static str, String> {
    match kind.unwrap_or("note") {
        "note" => Ok("NOTE"),
        "tip" => Ok("TIP"),
        "important" => Ok("IMPORTANT"),
        "warning" => Ok("WARNING"),
        "caution" => Ok("CAUTION"),
        other => Err(format!("Unsupported Tiptap callout kind {other}")),
    }
}

fn callout_content(
    node: &Node,
    context: &Context<'_>,
    depth: usize,
) -> Result<Vec<Element>, String> {
    model::allowed_attrs(&node.attrs, "callout", &["kind"])?;
    let label = callout_label(model::string_attr(&node.attrs, "kind")?)?;
    if node.content.is_empty() {
        return Err("Tiptap callouts require content".into());
    }
    let header = Element::Paragraph(Paragraph {
        content: vec![InlineContent::Text(TextSpan {
            text: label.into(),
            bold: true,
            ..Default::default()
        })],
        indent_left_twips: Some(360),
        ..Default::default()
    });
    let mut content = Vec::with_capacity(node.content.len() + 1);
    content.push(header);
    content.extend(indent_blocks(parse(&node.content, context, depth)?));
    Ok(content)
}

fn quote_content(node: &Node, context: &Context<'_>, depth: usize) -> Result<Vec<Element>, String> {
    model::no_attrs(&node.attrs, "blockquote")?;
    if node.content.is_empty() {
        return Err("Tiptap blockquotes require content".into());
    }
    Ok(indent_blocks(parse(&node.content, context, depth)?))
}

fn indent_blocks(mut content: Vec<Element>) -> Vec<Element> {
    for element in &mut content {
        match element {
            Element::Paragraph(paragraph) => {
                paragraph.indent_left_twips = Some(paragraph.indent_left_twips.unwrap_or(0) + 360);
            }
            Element::Heading(heading) => {
                heading.indent_left_twips = Some(heading.indent_left_twips.unwrap_or(0) + 360);
            }
            _ => {}
        }
    }
    content
}

fn code(node: &Node, context: &Context<'_>) -> Result<Element, String> {
    model::allowed_attrs(&node.attrs, "codeBlock", &["language"])?;
    let mut text = String::new();
    for child in &node.content {
        if child.kind != "text"
            || child.text.is_none()
            || !child.attrs.is_empty()
            || !child.content.is_empty()
            || !child.marks.is_empty()
        {
            return Err("Tiptap code blocks may contain only unmarked text".into());
        }
        text.push_str(child.text.as_deref().unwrap_or_default());
    }
    let language = model::string_attr(&node.attrs, "language")?.map(str::to_owned);
    if language
        .as_deref()
        .is_some_and(|lang| lang.eq_ignore_ascii_case("mermaid"))
        && !matches!(context.format, super::DocumentFormat::Text)
    {
        if let Some(images) = context.images {
            let destination = super::super::super::images::diagram_destination(&text);
            if let Ok(image) = images.get(&destination) {
                return Ok(Element::Image(office_oxide::ir::Image {
                    alt_text: Some("Mermaid diagram".into()),
                    data: Some(images.clone_bytes(&destination)?),
                    format: Some(office_oxide::ir::ImageFormat::Png),
                    display_width_emu: Some(u64::from(image.width) * 9_525),
                    display_height_emu: Some(u64::from(image.height) * 9_525),
                    pixel_width: Some(image.width),
                    pixel_height: Some(image.height),
                    ..Default::default()
                }));
            }
        }
    }
    Ok(Element::CodeBlock(CodeBlock {
        language,
        content: text,
    }))
}

fn block_math(node: &Node) -> Result<Element, String> {
    model::allowed_attrs(&node.attrs, "blockMath", &["latex"])?;
    if !node.content.is_empty() {
        return Err("Tiptap block math nodes must be empty".into());
    }
    let latex = model::string_attr(&node.attrs, "latex")?.unwrap_or_default();
    Ok(Element::Paragraph(Paragraph {
        content: super::super::super::math::to_ir_spans(latex, &TextSpan::default()),
        alignment: Some(ParagraphAlignment::Center),
        ..Default::default()
    }))
}

fn alignment(value: Option<&str>) -> Result<Option<ParagraphAlignment>, String> {
    value
        .map(|value| match value {
            "left" => Ok(ParagraphAlignment::Left),
            "center" => Ok(ParagraphAlignment::Center),
            "right" => Ok(ParagraphAlignment::Right),
            "justify" => Ok(ParagraphAlignment::Justify),
            _ => Err("Unsupported Tiptap text alignment".into()),
        })
        .transpose()
}

fn empty_block(node: &Node, element: Element) -> Result<Element, String> {
    model::no_attrs(&node.attrs, &node.kind)?;
    if !node.content.is_empty() {
        return Err(format!("Tiptap {} must be empty", node.kind));
    }
    Ok(element)
}

fn reject_block_extras(node: &Node) -> Result<(), String> {
    if node.text.is_some() || !node.marks.is_empty() {
        return Err(format!(
            "Tiptap block {} contains inline-only fields",
            node.kind
        ));
    }
    Ok(())
}

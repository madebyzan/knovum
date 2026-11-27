use office_oxide::ir::{CodeBlock, Element, Heading, Paragraph, ParagraphAlignment, TextBox};

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
        if node.kind == "blockquote" && matches!(context.format, super::DocumentFormat::Docx) {
            reject_block_extras(node)?;
            elements.extend(quote_content(node, context, depth + 1)?);
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
        "codeBlock" => code(node),
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

fn quote_content(node: &Node, context: &Context<'_>, depth: usize) -> Result<Vec<Element>, String> {
    model::no_attrs(&node.attrs, "blockquote")?;
    if node.content.is_empty() {
        return Err("Tiptap blockquotes require content".into());
    }
    let mut content = parse(&node.content, context, depth)?;
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
    Ok(content)
}

fn code(node: &Node) -> Result<Element, String> {
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
    Ok(Element::CodeBlock(CodeBlock {
        language: model::string_attr(&node.attrs, "language")?.map(str::to_owned),
        content: text,
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

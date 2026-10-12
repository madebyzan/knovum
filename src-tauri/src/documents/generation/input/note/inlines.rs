use office_oxide::ir::{InlineContent, TextSpan, UnderlineStyle, VerticalAlign};

use super::{colors, model};
use model::{Mark, Node};

pub(super) fn parse(nodes: &[Node]) -> Result<Vec<InlineContent>, String> {
    nodes.iter().map(inline).collect()
}

pub(super) fn plain(text: impl Into<String>) -> InlineContent {
    InlineContent::Text(TextSpan::plain(text))
}

fn inline(node: &Node) -> Result<InlineContent, String> {
    match node.kind.as_str() {
        "text" => text(node),
        "hardBreak" => {
            leaf(node)?;
            Ok(InlineContent::LineBreak)
        }
        kind => Err(format!("Unsupported Tiptap inline node {kind}")),
    }
}

fn text(node: &Node) -> Result<InlineContent, String> {
    model::no_attrs(&node.attrs, "text")?;
    if !node.content.is_empty() {
        return Err("Tiptap text nodes cannot contain child nodes".into());
    }
    let mut span = TextSpan::plain(
        node.text
            .as_ref()
            .ok_or("Tiptap text nodes require text")?
            .clone(),
    );
    for mark in &node.marks {
        apply_mark(&mut span, mark)?;
    }
    Ok(InlineContent::Text(span))
}

fn apply_mark(span: &mut TextSpan, mark: &Mark) -> Result<(), String> {
    match mark.kind.as_str() {
        "bold" => {
            model::no_attrs(&mark.attrs, "bold mark")?;
            span.bold = true;
        }
        "italic" => {
            model::no_attrs(&mark.attrs, "italic mark")?;
            span.italic = true;
        }
        "strike" => {
            model::no_attrs(&mark.attrs, "strike mark")?;
            span.strikethrough = true;
        }
        "underline" => {
            model::no_attrs(&mark.attrs, "underline mark")?;
            span.underline = Some(UnderlineStyle::Single);
        }
        "code" => {
            model::no_attrs(&mark.attrs, "code mark")?;
            span.font_name.get_or_insert_with(|| "monospace".into());
        }
        "subscript" => {
            model::no_attrs(&mark.attrs, "subscript mark")?;
            vertical(span, VerticalAlign::Subscript)?;
        }
        "superscript" => {
            model::no_attrs(&mark.attrs, "superscript mark")?;
            vertical(span, VerticalAlign::Superscript)?;
        }
        "highlight" => highlight(span, mark)?,
        "textStyle" => text_style(span, mark)?,
        "link" => link(span, mark)?,
        kind => return Err(format!("Unsupported Tiptap mark {kind}")),
    }
    Ok(())
}

fn vertical(span: &mut TextSpan, value: VerticalAlign) -> Result<(), String> {
    if span
        .vertical_align
        .as_ref()
        .is_some_and(|current| current != &value)
    {
        return Err("Tiptap text cannot be both subscript and superscript".into());
    }
    span.vertical_align = Some(value);
    Ok(())
}

fn highlight(span: &mut TextSpan, mark: &Mark) -> Result<(), String> {
    model::allowed_attrs(&mark.attrs, "highlight mark", &["color"])?;
    span.highlight = Some(match model::string_attr(&mark.attrs, "color")? {
        Some(color) => colors::parse(color)?,
        None => [255, 255, 0],
    });
    Ok(())
}

fn text_style(span: &mut TextSpan, mark: &Mark) -> Result<(), String> {
    model::allowed_attrs(
        &mark.attrs,
        "textStyle mark",
        &["fontSize", "color", "fontFamily"],
    )?;
    if let Some(size) = model::string_attr(&mark.attrs, "fontSize")? {
        span.font_size_half_pt = Some(font_size(size)?);
    }
    if let Some(color) = model::string_attr(&mark.attrs, "color")? {
        span.color = Some(colors::parse(color)?);
    }
    if let Some(font) = model::string_attr(&mark.attrs, "fontFamily")? {
        if font.is_empty() || font.len() > 128 {
            return Err("Tiptap font family must contain between 1 and 128 bytes".into());
        }
        span.font_name = Some(font.to_owned());
    }
    Ok(())
}

fn link(span: &mut TextSpan, mark: &Mark) -> Result<(), String> {
    model::allowed_attrs(
        &mark.attrs,
        "link mark",
        &["href", "target", "rel", "class"],
    )?;
    let href = model::string_attr(&mark.attrs, "href")?
        .filter(|href| !href.is_empty())
        .ok_or("Tiptap links require an href")?;
    super::super::super::links::validate(href)?;
    if !matches!(
        model::string_attr(&mark.attrs, "target")?,
        None | Some("_blank")
    ) || !matches!(
        model::string_attr(&mark.attrs, "rel")?,
        None | Some("noopener noreferrer nofollow")
    ) || model::string_attr(&mark.attrs, "class")?.is_some()
    {
        return Err("Unsupported Tiptap link presentation attributes".into());
    }
    if span
        .hyperlink
        .as_deref()
        .is_some_and(|existing| existing != href)
    {
        return Err("Tiptap text contains conflicting link marks".into());
    }
    span.hyperlink = Some(href.to_owned());
    Ok(())
}

fn font_size(value: &str) -> Result<u32, String> {
    let (number, factor) = value
        .strip_suffix("pt")
        .map(|value| (value, 2.0))
        .or_else(|| value.strip_suffix("px").map(|value| (value, 1.5)))
        .ok_or("Tiptap font size must use pt or px")?;
    let points = number
        .trim()
        .parse::<f64>()
        .map_err(|_| "Invalid Tiptap font size")?
        * factor;
    if !points.is_finite() || !(1.0..=1152.0).contains(&points) {
        return Err("Tiptap font size is outside the supported range".into());
    }
    Ok(points.round() as u32)
}

fn leaf(node: &Node) -> Result<(), String> {
    model::no_attrs(&node.attrs, &node.kind)?;
    if node.text.is_some() || !node.content.is_empty() || !node.marks.is_empty() {
        return Err(format!("Tiptap {} must be an empty leaf node", node.kind));
    }
    Ok(())
}

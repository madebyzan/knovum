use office_oxide::ir::{Element, Image, ImageFormat, Paragraph};

use super::{inlines, model, Context};
use model::Node;

pub(super) fn image(node: &Node, context: &Context<'_>) -> Result<Element, String> {
    model::allowed_attrs(
        &node.attrs,
        "image",
        &["src", "assetPath", "alt", "title", "width", "height"],
    )?;
    if !node.content.is_empty() {
        return Err("Tiptap image nodes cannot contain child nodes".into());
    }
    let src = required_string(node, "src", "Tiptap images require a source identity")?;
    required_string(
        node,
        "assetPath",
        "Import document images into this Space before conversion",
    )?;
    let alt = model::string_attr(&node.attrs, "alt")?.unwrap_or("");
    let title = model::string_attr(&node.attrs, "title")?;
    if title.is_some_and(|title| !alt.is_empty() && title != alt) {
        return Err("Tiptap image title cannot be preserved separately from alt text".into());
    }
    let description = if alt.is_empty() {
        title.unwrap_or("")
    } else {
        alt
    };
    if matches!(context.format, super::DocumentFormat::Text) {
        return Ok(Element::Paragraph(Paragraph {
            content: vec![inlines::plain(format!("[Image: {description}]"))],
            ..Default::default()
        }));
    }
    let image = context
        .images
        .ok_or("Document image resources were not loaded")?
        .get(src)?;
    let (width, height) = dimensions(node, image.width, image.height)?;
    Ok(Element::Image(Image {
        alt_text: (!description.is_empty()).then(|| description.to_owned()),
        data: Some(
            context
                .images
                .ok_or("Document image resources were not loaded")?
                .clone_bytes(src)?,
        ),
        format: Some(ImageFormat::Png),
        display_width_emu: Some(u64::from(width) * 9_525),
        display_height_emu: Some(u64::from(height) * 9_525),
        pixel_width: Some(image.width),
        pixel_height: Some(image.height),
        ..Default::default()
    }))
}

fn dimensions(node: &Node, width: u32, height: u32) -> Result<(u32, u32), String> {
    let requested_width = dimension(node, "width")?;
    let requested_height = dimension(node, "height")?;
    Ok(match (requested_width, requested_height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => (
            w,
            (((u64::from(w) * u64::from(height)) / u64::from(width)) as u32).max(1),
        ),
        (None, Some(h)) => (
            (((u64::from(h) * u64::from(width)) / u64::from(height)) as u32).max(1),
            h,
        ),
        (None, None) => (width, height),
    })
}

fn dimension(node: &Node, key: &str) -> Result<Option<u32>, String> {
    match node.attrs.get(key) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(value)) => value
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| (1..=16_384).contains(value))
            .map(Some)
            .ok_or_else(|| format!("Tiptap image {key} is outside the supported range")),
        _ => Err(format!("Tiptap image {key} must be an integer or null")),
    }
}

fn required_string<'a>(node: &'a Node, key: &str, error: &str) -> Result<&'a str, String> {
    model::string_attr(&node.attrs, key)?
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error.into())
}

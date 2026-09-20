use std::collections::{HashMap, HashSet};

use super::contract::{SpaceEditorDocument, CanvasNodeLayoutV1};

pub(in crate::spaces) fn validate_document(document: &SpaceEditorDocument) -> Result<(), String> {
    if document.version != 1 {
        return Err("Unsupported Canvas document version".into());
    }
    validate_number("viewport.x", document.viewport.x, false)?;
    validate_number("viewport.y", document.viewport.y, false)?;
    validate_number("viewport.zoom", document.viewport.zoom, true)?;
    if document.nodes.len() > 100_000 {
        return Err("Canvas document contains too many nodes".into());
    }
    let mut nodes = HashMap::with_capacity(document.nodes.len());
    for (index, node) in document.nodes.iter().enumerate() {
        if node.node_id.is_empty() || nodes.insert(node.node_id.as_str(), node).is_some() {
            return Err(format!(
                "nodes[{index}].nodeId must be non-empty and unique"
            ));
        }
        validate_frame(index, node)?;
    }
    validate_relation_styles(document)?;
    Ok(())
}

fn validate_relation_styles(document: &SpaceEditorDocument) -> Result<(), String> {
    let styles = document.relation_styles.as_deref().unwrap_or_default();
    if styles.len() > 100_000 {
        return Err("Canvas document contains too many relation styles".into());
    }
    let mut relation_ids = HashSet::with_capacity(styles.len());
    for (index, style) in styles.iter().enumerate() {
        if style.relation_id.is_empty() || !relation_ids.insert(style.relation_id.as_str()) {
            return Err(format!(
                "relationStyles[{index}].relationId must be non-empty and unique"
            ));
        }
    }
    Ok(())
}

fn validate_frame(index: usize, node: &CanvasNodeLayoutV1) -> Result<(), String> {
    if node
        .file_subpath
        .as_deref()
        .is_some_and(|subpath| !subpath.starts_with('#'))
    {
        return Err(format!("nodes[{index}].fileSubpath must start with #"));
    }
    if node
        .appearance
        .as_ref()
        .and_then(|appearance| appearance.typography_scale)
        .is_some_and(|scale| !scale.is_finite() || !(0.25..=4.0).contains(&scale))
    {
        return Err(format!(
            "nodes[{index}].appearance.typographyScale must be between 0.25 and 4"
        ));
    }
    if node
        .appearance
        .as_ref()
        .and_then(|appearance| appearance.font_size)
        .is_some_and(|size| !size.is_finite() || !(2.5..=576.0).contains(&size))
    {
        return Err(format!(
            "nodes[{index}].appearance.fontSize must be between 2.5 and 576"
        ));
    }
    if let Some(appearance) = node.appearance.as_ref() {
        if appearance.fill.is_some() && appearance.custom_fill.is_some() {
            return Err(format!(
                "nodes[{index}].appearance.fill and customFill are mutually exclusive"
            ));
        }
        if appearance
            .custom_fill
            .as_deref()
            .is_some_and(|color| !is_hex_color(color))
        {
            return Err(format!(
                "nodes[{index}].appearance.customFill must be a #RRGGBB color"
            ));
        }
    }
    for (field, value, positive) in [
        ("x", node.frame.x, false),
        ("y", node.frame.y, false),
        ("width", node.frame.width, true),
        ("height", node.frame.height, true),
        ("rotation", node.frame.rotation, false),
    ] {
        validate_number(&format!("nodes[{index}].frame.{field}"), value, positive)?;
    }
    Ok(())
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.as_bytes()[1..]
            .iter()
            .all(|byte| byte.is_ascii_hexdigit())
}

fn validate_number(path: &str, value: f64, positive: bool) -> Result<(), String> {
    if !value.is_finite() || (positive && value <= 0.0) {
        return Err(format!("{path} must be a valid canvas number"));
    }
    Ok(())
}

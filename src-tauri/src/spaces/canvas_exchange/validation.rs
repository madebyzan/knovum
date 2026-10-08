use std::collections::HashSet;

use super::contracts::JsonCanvasDocumentV1;

pub(crate) fn validate(document: &JsonCanvasDocumentV1) -> Result<(), String> {
    let mut node_ids = HashSet::with_capacity(document.nodes.len());
    for (index, node) in document.nodes.iter().enumerate() {
        if !node_ids.insert(node.frame().id.as_str()) {
            return Err(format!("nodes[{index}].id must be unique"));
        }
        if node.subpath().is_some_and(|value| !value.starts_with('#')) {
            return Err(format!("nodes[{index}].subpath must start with #"));
        }
        validate_frame(index, node.frame())?;
        if let Some(color) = node.color() {
            validate_color(&format!("nodes[{index}].color"), color)?;
        }
    }

    let mut edge_ids = HashSet::with_capacity(document.edges.len());
    for (index, edge) in document.edges.iter().enumerate() {
        if !edge_ids.insert(edge.id.as_str()) {
            return Err(format!("edges[{index}].id must be unique"));
        }
        if !node_ids.contains(edge.from_node.as_str()) {
            return Err(format!("edges[{index}].fromNode does not reference a node"));
        }
        if !node_ids.contains(edge.to_node.as_str()) {
            return Err(format!("edges[{index}].toNode does not reference a node"));
        }
        if let Some(color) = edge.color.as_deref() {
            validate_color(&format!("edges[{index}].color"), color)?;
        }
    }
    Ok(())
}

fn validate_frame(index: usize, frame: &super::contracts::NodeFrameV1) -> Result<(), String> {
    const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
    if frame.id.is_empty() {
        return Err(format!("nodes[{index}].id must not be empty"));
    }
    for (field, value) in [
        ("x", frame.x),
        ("y", frame.y),
        ("width", frame.width),
        ("height", frame.height),
    ] {
        if value.fract() != 0.0 {
            return Err(format!("nodes[{index}].{field} must be an integer"));
        }
        if !value.is_finite() || value.abs() > MAX_SAFE_INTEGER {
            return Err(format!("nodes[{index}].{field} must be a safe integer"));
        }
    }
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return Err(format!(
            "nodes[{index}] width and height must be positive integers"
        ));
    }
    Ok(())
}

fn validate_color(path: &str, color: &str) -> Result<(), String> {
    let preset = matches!(color, "1" | "2" | "3" | "4" | "5" | "6");
    let hex = color.strip_prefix('#').is_some_and(|digits| {
        digits.len() == 6 && digits.chars().all(|digit| digit.is_ascii_hexdigit())
    });
    if preset || hex {
        Ok(())
    } else {
        Err(format!(
            "{path} must be a JSON Canvas preset or hexadecimal color"
        ))
    }
}

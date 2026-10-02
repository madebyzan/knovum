use serde_json::{Map, Value};

use super::super::{
    json_value::validate_json_object,
    scalars::{required_string, required_string_allow_empty},
    validation::reject_unknown,
};
use super::MAX_TEXT;

const MAX_TIPTAP_DEPTH: usize = 64;

pub(super) fn validate_canvas_text(object: &Map<String, Value>) -> Result<(), String> {
    required_string_allow_empty(object, "plainText", MAX_TEXT)?;
    let root = object
        .get("tiptapContent")
        .filter(|value| value.is_object())
        .ok_or_else(|| "Canvas text tiptapContent must be an object".to_string())?;
    if root.get("type").and_then(Value::as_str) != Some("doc") {
        return Err("Tiptap root must be doc".into());
    }
    let mut nodes = 0usize;
    validate_node(root, 0, &mut nodes)
}

fn validate_node(value: &Value, depth: usize, nodes: &mut usize) -> Result<(), String> {
    if depth > MAX_TIPTAP_DEPTH {
        return Err("Tiptap content nesting is too deep".into());
    }
    *nodes += 1;
    if *nodes > 100_000 {
        return Err("Tiptap content contains too many nodes".into());
    }
    let object = value
        .as_object()
        .ok_or_else(|| "Tiptap node must be an object".to_string())?;
    reject_unknown(
        object,
        ["type", "attrs", "content", "marks", "text"],
        "Tiptap node",
    )?;
    required_string(object, "type", 64)?;
    validate_text(object)?;
    validate_attrs(object)?;
    validate_marks(object)?;
    validate_content(object, depth, nodes)
}

fn validate_text(object: &Map<String, Value>) -> Result<(), String> {
    if let Some(text) = object.get("text") {
        let text = text
            .as_str()
            .ok_or_else(|| "Tiptap text must be a string".to_string())?;
        if text.encode_utf16().count() > MAX_TEXT {
            return Err("Tiptap text is too large".into());
        }
    }
    Ok(())
}

fn validate_attrs(object: &Map<String, Value>) -> Result<(), String> {
    if let Some(attrs) = object.get("attrs") {
        validate_json_object(attrs, "Tiptap node attrs")?;
    }
    Ok(())
}

fn validate_marks(object: &Map<String, Value>) -> Result<(), String> {
    let Some(marks) = object.get("marks") else {
        return Ok(());
    };
    let marks = marks
        .as_array()
        .ok_or_else(|| "Tiptap marks must be an array".to_string())?;
    if marks.len() > 256 {
        return Err("Tiptap node contains too many marks".into());
    }
    for mark in marks {
        let mark = mark
            .as_object()
            .ok_or_else(|| "Tiptap mark must be an object".to_string())?;
        reject_unknown(mark, ["type", "attrs"], "Tiptap mark")?;
        required_string(mark, "type", 64)?;
        if let Some(attrs) = mark.get("attrs") {
            validate_json_object(attrs, "Tiptap mark attrs")?;
        }
    }
    Ok(())
}

fn validate_content(
    object: &Map<String, Value>,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), String> {
    let Some(content) = object.get("content") else {
        return Ok(());
    };
    let content = content
        .as_array()
        .ok_or_else(|| "Tiptap content must be an array".to_string())?;
    if content.len() > 100_000 {
        return Err("Tiptap node contains too many children".into());
    }
    for child in content {
        validate_node(child, depth + 1, nodes)?;
    }
    Ok(())
}

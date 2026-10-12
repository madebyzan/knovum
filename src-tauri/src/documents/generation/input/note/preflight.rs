use serde_json::Value;

use super::super::super::{document::NoteSource, markdown};

const MAX_TEXT_BYTES: usize = 4 * 1024 * 1024;
const MAX_ELEMENTS: usize = 200_000;
const MAX_DEPTH: usize = 64;

pub(super) fn validate(source: &NoteSource) -> Result<(), String> {
    if source.citations.len() > 4096 {
        return Err("Document export exceeds 4096 citations".into());
    }
    let mut text_bytes = 0usize;
    for text in std::iter::once(source.title.as_str()).chain(
        source
            .citations
            .iter()
            .flat_map(|citation| [citation.title.as_str(), citation.quote.as_str()]),
    ) {
        add_text(text, &mut text_bytes)?;
    }
    validate_tree(&source.content, &mut text_bytes)
}

fn validate_tree(root: &Value, text_bytes: &mut usize) -> Result<(), String> {
    let mut elements = 0usize;
    let mut stack = vec![(root, 0usize)];
    while let Some((node, depth)) = stack.pop() {
        if depth > MAX_DEPTH {
            return Err("Document export nesting exceeds 64 levels".into());
        }
        elements = elements.saturating_add(1);
        let object = match node.as_object() {
            Some(object) => object,
            None => continue,
        };
        if let Some(text) = object.get("text").and_then(Value::as_str) {
            add_text(text, text_bytes)?;
        }
        if let Some(attrs) = object.get("attrs") {
            add_attribute_text(attrs, text_bytes)?;
        }
        if let Some(marks) = object.get("marks").and_then(Value::as_array) {
            elements = elements.saturating_add(marks.len());
            for mark in marks {
                if let Some(attrs) = mark.get("attrs") {
                    add_attribute_text(attrs, text_bytes)?;
                }
            }
        }
        if elements > MAX_ELEMENTS {
            return Err("Document export exceeds 200000 Tiptap elements".into());
        }
        if let Some(children) = object.get("content").and_then(Value::as_array) {
            stack.extend(children.iter().map(|child| (child, depth + 1)));
        }
    }
    Ok(())
}

fn add_attribute_text(value: &Value, total: &mut usize) -> Result<(), String> {
    let mut values = vec![(value, 0usize)];
    while let Some((value, depth)) = values.pop() {
        if depth > MAX_DEPTH {
            return Err("Document export attribute nesting exceeds 64 levels".into());
        }
        match value {
            Value::String(text) => add_text(text, total)?,
            Value::Array(children) => {
                values.extend(children.iter().map(|child| (child, depth + 1)))
            }
            Value::Object(children) => {
                values.extend(children.values().map(|child| (child, depth + 1)))
            }
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
    }
    Ok(())
}

fn add_text(text: &str, total: &mut usize) -> Result<(), String> {
    markdown::validate_text(text)?;
    *total = total.saturating_add(text.len());
    if *total > MAX_TEXT_BYTES {
        return Err("Document export textual content exceeds 4 MiB".into());
    }
    Ok(())
}

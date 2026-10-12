use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Node {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub attrs: Map<String, Value>,
    #[serde(default)]
    pub content: Vec<Node>,
    #[serde(default)]
    pub marks: Vec<Mark>,
    pub text: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Mark {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub attrs: Map<String, Value>,
}

pub(super) fn no_attrs(attrs: &Map<String, Value>, owner: &str) -> Result<(), String> {
    if attrs.is_empty() {
        Ok(())
    } else {
        Err(format!("Unsupported Tiptap attributes on {owner}"))
    }
}

pub(super) fn allowed_attrs(
    attrs: &Map<String, Value>,
    owner: &str,
    allowed: &[&str],
) -> Result<(), String> {
    if let Some(key) = attrs.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(format!("Unsupported Tiptap attribute {key} on {owner}"));
    }
    Ok(())
}

pub(super) fn string_attr<'a>(
    attrs: &'a Map<String, Value>,
    key: &str,
) -> Result<Option<&'a str>, String> {
    match attrs.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(format!("Tiptap attribute {key} must be a string or null")),
    }
}

pub(super) fn u32_attr(attrs: &Map<String, Value>, key: &str, default: u32) -> Result<u32, String> {
    match attrs.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Number(value)) => value
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| format!("Tiptap attribute {key} must be a positive integer")),
        Some(_) => Err(format!("Tiptap attribute {key} must be an integer")),
    }
}

pub(super) fn bool_attr(
    attrs: &Map<String, Value>,
    key: &str,
    default: bool,
) -> Result<bool, String> {
    match attrs.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(format!("Tiptap attribute {key} must be a boolean")),
    }
}

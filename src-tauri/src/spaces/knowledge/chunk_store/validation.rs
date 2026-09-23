use serde_json::Value;

use crate::spaces::knowledge::{indexing::MAX_KNOWLEDGE_PASSAGE_BYTES, locator};

pub(in crate::spaces) fn validate_values(
    origin: &str,
    kind: &str,
    text: &str,
    locator_value: &Value,
    metadata: Option<&Value>,
    expected_revision: &str,
) -> Result<(), String> {
    validate_origin(origin)?;
    validate_kind(kind)?;
    if text.trim().is_empty() {
        return Err("Knowledge chunk text cannot be empty".into());
    }
    if text.len() > MAX_KNOWLEDGE_PASSAGE_BYTES {
        return Err(format!(
            "Knowledge passage exceeds the {} byte durable embedding limit",
            MAX_KNOWLEDGE_PASSAGE_BYTES
        ));
    }
    let source_revision = locator_value
        .get("sourceRevision")
        .and_then(Value::as_str)
        .ok_or_else(|| "Knowledge chunk locator is missing sourceRevision".to_owned())?;
    if source_revision != expected_revision {
        return Err("Knowledge chunk revision does not match the owning node revision".into());
    }
    locator::validate(locator_value, Some(source_revision))?;
    let empty_metadata = Value::Object(Default::default());
    validate_metadata(metadata.unwrap_or(&empty_metadata))
}

pub(in crate::spaces) fn validate_origin(origin: &str) -> Result<(), String> {
    match origin {
        "source" | "media" | "derived" => Ok(()),
        _ => Err("Only supplemental knowledge origins can be replaced explicitly".to_owned()),
    }
}

pub(in crate::spaces) fn validate_kind(kind: &str) -> Result<(), String> {
    match kind {
        "node" | "text" | "heading" | "page" | "slide" | "sheet" | "table" | "transcript"
        | "chapter" | "code" | "image_description" | "video_cover" => Ok(()),
        _ => Err("Unsupported knowledge chunk kind".to_owned()),
    }
}

pub(in crate::spaces) fn validate_metadata(metadata: &Value) -> Result<(), String> {
    let object = metadata
        .as_object()
        .ok_or_else(|| "Knowledge chunk metadata must be an object".to_owned())?;
    if object.len() > 256 {
        return Err("Knowledge chunk metadata may contain at most 256 fields".to_owned());
    }
    for (key, value) in object {
        if key.is_empty() || key.encode_utf16().count() > 256 {
            return Err(
                "Knowledge chunk metadata keys must contain 1 to 256 characters".to_owned(),
            );
        }
        if !matches!(
            value,
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
        ) {
            return Err("Knowledge chunk metadata values must be primitive JSON values".to_owned());
        }
        if value
            .as_str()
            .is_some_and(|text| text.encode_utf16().count() > 16_000)
        {
            return Err(
                "Knowledge chunk metadata string values may contain at most 16000 characters"
                    .to_owned(),
            );
        }
    }
    Ok(())
}

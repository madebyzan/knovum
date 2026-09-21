use serde_json::{Map, Value};

pub(in crate::spaces::knowledge::node_contracts) fn required_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    max: usize,
) -> Result<&'a str, String> {
    let text = object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Space node data.{key} must be a non-empty string"))?;
    if text.encode_utf16().count() > max {
        return Err(format!("Space node data.{key} exceeds {max} characters"));
    }
    Ok(text)
}

pub(in crate::spaces::knowledge::node_contracts) fn required_string_allow_empty<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    max: usize,
) -> Result<&'a str, String> {
    let text = object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Space node data.{key} must be a string"))?;
    if text.encode_utf16().count() > max {
        return Err(format!("Space node data.{key} exceeds {max} characters"));
    }
    Ok(text)
}

pub(in crate::spaces::knowledge::node_contracts) fn optional_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    max: usize,
    allow_empty: bool,
) -> Result<Option<&'a str>, String> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    let text = value
        .as_str()
        .ok_or_else(|| format!("Space node data.{key} must be a string"))?;
    if !allow_empty && text.trim().is_empty() {
        return Err(format!("Space node data.{key} must be non-empty"));
    }
    if text.encode_utf16().count() > max {
        return Err(format!("Space node data.{key} exceeds {max} characters"));
    }
    Ok(Some(text))
}

pub(in crate::spaces::knowledge::node_contracts) fn optional_string_allow_empty(
    object: &Map<String, Value>,
    key: &str,
    max: usize,
) -> Result<(), String> {
    optional_string(object, key, max, true).map(|_| ())
}

pub(in crate::spaces::knowledge::node_contracts) fn optional_string_array(
    object: &Map<String, Value>,
    key: &str,
    max_entries: usize,
    max_chars: usize,
) -> Result<(), String> {
    let Some(value) = object.get(key) else {
        return Ok(());
    };
    let values = value
        .as_array()
        .ok_or_else(|| format!("Space node data.{key} must be an array"))?;
    if values.len() > max_entries {
        return Err(format!(
            "Space node data.{key} exceeds {max_entries} entries"
        ));
    }
    for value in values {
        let text = value
            .as_str()
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| format!("Space node data.{key} must contain non-empty strings"))?;
        if text.encode_utf16().count() > max_chars {
            return Err(format!("Space node data.{key} contains an oversized entry"));
        }
    }
    Ok(())
}

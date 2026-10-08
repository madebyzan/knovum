use serde_json::{Map, Value};

use super::{MAX_HEADING_PATH_ITEMS, MAX_LOCATION_TEXT_CHARS};

pub(super) fn ensure_keys(
    object: &Map<String, Value>,
    allowed: &[&str],
    label: &str,
) -> Result<(), String> {
    if let Some(key) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(format!("{label} field '{key}' is not supported"));
    }
    Ok(())
}

pub(super) fn required_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<&'a str, String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{label} must be a non-empty string"))
}

pub(super) fn validate_text_limit(
    value: &str,
    max_chars: usize,
    label: &str,
) -> Result<(), String> {
    if value.encode_utf16().count() > max_chars {
        return Err(format!(
            "{label} exceeds the maximum of {max_chars} characters"
        ));
    }
    Ok(())
}

pub(super) fn required_string_limited<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
    max_chars: usize,
) -> Result<&'a str, String> {
    let value = required_string(object, key, label)?;
    validate_text_limit(value, max_chars, label)?;
    Ok(value)
}

pub(super) fn optional_string_limited(
    object: &Map<String, Value>,
    key: &str,
    label: &str,
    max_chars: usize,
) -> Result<(), String> {
    let Some(value) = object.get(key) else {
        return Ok(());
    };
    let value = value
        .as_str()
        .ok_or_else(|| format!("{label} must be a string"))?;
    validate_text_limit(value, max_chars, label)
}

pub(super) fn required_string_array(
    object: &Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<(), String> {
    let values = object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{label} must be an array"))?;
    if values.len() > MAX_HEADING_PATH_ITEMS {
        return Err(format!("{label} exceeds {MAX_HEADING_PATH_ITEMS} entries"));
    }
    for value in values {
        let text = value
            .as_str()
            .ok_or_else(|| format!("{label} must contain strings"))?;
        validate_text_limit(text, MAX_LOCATION_TEXT_CHARS, label)?;
    }
    Ok(())
}

pub(super) fn required_nonnegative_i64(
    object: &Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<i64, String> {
    let value = object
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{label} must be an integer"))?;
    if value < 0 {
        return Err(format!("{label} must be non-negative"));
    }
    Ok(value)
}

pub(super) fn required_positive_i64(
    object: &Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<i64, String> {
    let value = required_nonnegative_i64(object, key, label)?;
    if value == 0 {
        return Err(format!("{label} must be positive"));
    }
    Ok(value)
}

pub(super) fn optional_nonnegative_i64(
    object: &Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<Option<i64>, String> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    let value = value
        .as_i64()
        .ok_or_else(|| format!("{label} must be an integer"))?;
    if value < 0 {
        return Err(format!("{label} must be non-negative"));
    }
    Ok(Some(value))
}

pub(super) fn required_f64(
    object: &Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<f64, String> {
    object
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .ok_or_else(|| format!("{label} must be a finite number"))
}

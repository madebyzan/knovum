use serde_json::Value;

use super::scalars::is_javascript_safe_number;

const MAX_JSON_DEPTH: usize = 64;
const MAX_JSON_VALUES: usize = 100_000;
const MAX_JSON_CONTAINER_ITEMS: usize = 10_000;
const MAX_JSON_KEY_UTF16: usize = 128;

pub(super) fn validate_json_object(value: &Value, label: &str) -> Result<(), String> {
    if !value.is_object() {
        return Err(format!("{label} must be an object"));
    }
    let mut values = 0usize;
    validate_json_value(value, 0, &mut values, label)
}

fn validate_json_value(
    value: &Value,
    depth: usize,
    values: &mut usize,
    label: &str,
) -> Result<(), String> {
    if depth > MAX_JSON_DEPTH {
        return Err(format!("{label} nesting is too deep"));
    }
    *values += 1;
    if *values > MAX_JSON_VALUES {
        return Err(format!("{label} contains too many JSON values"));
    }
    match value {
        Value::Null | Value::Bool(_) | Value::String(_) => Ok(()),
        Value::Number(_) => {
            if is_javascript_safe_number(value) {
                Ok(())
            } else {
                Err(format!(
                    "{label} numbers must be finite and integral values must be JavaScript-safe"
                ))
            }
        }
        Value::Array(items) => {
            if items.len() > MAX_JSON_CONTAINER_ITEMS {
                return Err(format!(
                    "{label} array exceeds {MAX_JSON_CONTAINER_ITEMS} items"
                ));
            }
            for item in items {
                validate_json_value(item, depth + 1, values, label)?;
            }
            Ok(())
        }
        Value::Object(object) => {
            if object.len() > MAX_JSON_CONTAINER_ITEMS {
                return Err(format!(
                    "{label} object exceeds {MAX_JSON_CONTAINER_ITEMS} fields"
                ));
            }
            for (key, item) in object {
                if key.encode_utf16().count() > MAX_JSON_KEY_UTF16 {
                    return Err(format!(
                        "{label} JSON key exceeds {MAX_JSON_KEY_UTF16} characters"
                    ));
                }
                validate_json_value(item, depth + 1, values, label)?;
            }
            Ok(())
        }
    }
}

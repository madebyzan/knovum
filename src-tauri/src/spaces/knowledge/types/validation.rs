use std::collections::HashSet;

use serde_json::Value;

use super::ObjectTypeInput;

pub(super) fn validate_input(input: &ObjectTypeInput) -> Result<(), String> {
    validate_custom_type_definition(
        &input.id,
        &input.name,
        &input.icon,
        &input.description,
        &input.fields,
    )
}

pub(crate) fn validate_custom_type_definition(
    id: &str,
    name: &str,
    icon: &str,
    description: &str,
    fields: &Value,
) -> Result<(), String> {
    validate_required_text(id, 128, "Knowledge Type id")?;
    validate_required_text(name, 128, "Knowledge Type name")?;
    validate_required_text(icon, 128, "Knowledge Type icon")?;
    validate_text(description, 4_096, "Knowledge Type description")?;
    if id.starts_with("system:") {
        return Err("Custom Knowledge Type ids cannot use the reserved system: prefix".into());
    }
    validate_fields(fields)
}

fn validate_fields(value: &Value) -> Result<(), String> {
    let fields = value
        .as_array()
        .ok_or_else(|| "Knowledge Type fields must be an array".to_string())?;
    if fields.len() > 64 {
        return Err("Knowledge Type cannot define more than 64 fields".into());
    }
    let mut keys = HashSet::new();
    for field in fields {
        validate_field(field, &mut keys)?;
    }
    Ok(())
}

fn validate_field(field: &Value, keys: &mut HashSet<String>) -> Result<(), String> {
    let object = field
        .as_object()
        .ok_or_else(|| "Knowledge Type field must be an object".to_string())?;
    let allowed: HashSet<&str> = ["key", "label", "kind", "required", "options"]
        .into_iter()
        .collect();
    if let Some(name) = object.keys().find(|name| !allowed.contains(name.as_str())) {
        return Err(format!(
            "Knowledge Type field property '{name}' is not supported"
        ));
    }
    let key = field
        .get("key")
        .and_then(Value::as_str)
        .filter(|item| !item.is_empty())
        .ok_or_else(|| "Knowledge Type field key is required".to_string())?;
    validate_required_text(key, 128, "Knowledge Type field key")?;
    if !keys.insert(key.to_owned()) {
        return Err(format!("Duplicate Knowledge Type field '{key}'"));
    }
    validate_field_contract(field, key)
}

fn validate_field_contract(field: &Value, key: &str) -> Result<(), String> {
    let label = field
        .get("label")
        .and_then(Value::as_str)
        .filter(|item| !item.is_empty())
        .ok_or_else(|| format!("Knowledge Type field '{key}' label is required"))?;
    validate_required_text(label, 128, "Knowledge Type field label")?;
    if field.get("required").is_some_and(|item| !item.is_boolean()) {
        return Err(format!(
            "Knowledge Type field '{key}' required flag must be boolean"
        ));
    }
    let kind = field
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Knowledge Type field '{key}' requires a kind"))?;
    if !matches!(
        kind,
        "text" | "number" | "boolean" | "date" | "node-ref" | "enum"
    ) {
        return Err(format!("Unsupported Knowledge Type field kind '{kind}'"));
    }
    match (kind, field.get("options")) {
        ("enum", Some(options)) => validate_enum_options(key, options),
        ("enum", None) => Err(format!("Enum field '{key}' requires options")),
        (_, Some(_)) => Err(format!("Only enum field '{key}' may define options")),
        _ => Ok(()),
    }
}

fn validate_enum_options(key: &str, value: &Value) -> Result<(), String> {
    let options = value
        .as_array()
        .ok_or_else(|| format!("Enum field '{key}' options must be an array"))?;
    if options.is_empty() || options.len() > 64 {
        return Err(format!(
            "Enum field '{key}' requires between 1 and 64 options"
        ));
    }
    let mut unique = HashSet::new();
    for option in options {
        let text = option
            .as_str()
            .filter(|item| !item.trim().is_empty())
            .ok_or_else(|| format!("Enum field '{key}' options must be non-empty strings"))?;
        validate_required_text(text, 128, "Enum option")?;
        if !unique.insert(text) {
            return Err(format!(
                "Enum field '{key}' contains duplicate option '{text}'"
            ));
        }
    }
    Ok(())
}

fn validate_required_text(value: &str, max: usize, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{label} is required"));
    }
    validate_text(value, max, label)
}

fn validate_text(value: &str, max: usize, label: &str) -> Result<(), String> {
    if value.encode_utf16().count() > max {
        return Err(format!("{label} exceeds {max} characters"));
    }
    Ok(())
}

use serde_json::Value;

pub(in crate::spaces) fn collect_node_ids(
    fields_raw: Option<&str>,
    payload: &Value,
) -> Result<Vec<String>, String> {
    let Some(fields_raw) = fields_raw else {
        return Ok(Vec::new());
    };
    let fields: Value = serde_json::from_str(fields_raw).map_err(crate::spaces::db::json_error)?;
    let fields = fields
        .as_array()
        .ok_or_else(|| "Knowledge Type fields are invalid".to_string())?;
    let properties = payload
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "Knowledge object properties are invalid".to_string())?;
    let mut references = Vec::new();
    for field in fields {
        if field.get("kind").and_then(Value::as_str) != Some("node-ref") {
            continue;
        }
        let key = field
            .get("key")
            .and_then(Value::as_str)
            .ok_or_else(|| "Knowledge Type node-ref field key is invalid".to_string())?;
        if let Some(target) = properties.get(key).and_then(Value::as_str) {
            references.push(target.to_owned());
        }
    }
    Ok(references)
}

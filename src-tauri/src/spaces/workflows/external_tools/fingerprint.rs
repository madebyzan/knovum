use std::collections::BTreeMap;

use serde_json::Value;
use sha2::{Digest, Sha256};

pub(super) fn calculate(
    server_identity: &Value,
    tool_name: &str,
    input_schema: &Value,
    output_schema: Option<&Value>,
) -> Result<String, String> {
    let value = serde_json::json!({
        "serverIdentity": canonical(server_identity),
        "toolName": tool_name,
        "inputSchema": canonical(input_schema),
        "outputSchema": output_schema.map(canonical),
    });
    let encoded = serde_json::to_vec(&value).map_err(crate::spaces::db::json_error)?;
    Ok(format!("{:x}", Sha256::digest(encoded)))
}

fn canonical(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(canonical).collect()),
        Value::Object(object) => {
            let sorted = object
                .iter()
                .map(|(key, value)| (key.clone(), canonical(value)))
                .collect::<BTreeMap<_, _>>();
            serde_json::to_value(sorted)
                .expect("canonical JSON contains only repository-owned JSON values")
        }
        value => value.clone(),
    }
}

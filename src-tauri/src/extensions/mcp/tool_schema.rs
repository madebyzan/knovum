//! The MCP tool listing contract (`Tool.inputSchema` in schema.ts) is a JSON Schema object whose
//! `properties` values are objects. JSON Schema itself also allows boolean subschemas — `true`
//! accepts anything, `false` nothing — and generators emit them for untyped values. MCP clients
//! built on the reference SDK validate the listing as a whole, so one boolean property schema
//! makes them drop every tool of that server. Conforming rewrites each boolean property schema
//! into its exact object equivalent; the accepted instances are unchanged.

use serde_json::{json, Map, Value};

pub(crate) fn conform(schema: &mut Map<String, Value>) {
    let Some(Value::Object(properties)) = schema.get_mut("properties") else {
        return;
    };
    for property in properties.values_mut() {
        if let Value::Bool(accepts) = *property {
            *property = if accepts {
                json!({})
            } else {
                json!({ "not": {} })
            };
        }
    }
}

#[cfg(test)]
#[path = "tests/tool_schema.rs"]
mod tests;

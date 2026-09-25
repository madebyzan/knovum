use std::collections::HashSet;

use serde_json::Value;

pub(crate) fn source_node_ids(value: &Value) -> Result<Vec<String>, String> {
    fn push(id: &str, ids: &mut Vec<String>, seen: &mut HashSet<String>) {
        if seen.insert(id.to_owned()) {
            ids.push(id.to_owned());
        }
    }

    fn visit(value: &Value, ids: &mut Vec<String>, seen: &mut HashSet<String>) {
        match value {
            Value::Array(values) => values.iter().for_each(|value| visit(value, ids, seen)),
            Value::Object(values) => {
                for key in ["nodeId", "sourceNodeId"] {
                    if let Some(id) = values.get(key).and_then(Value::as_str) {
                        push(id, ids, seen);
                    }
                }
                for key in ["nodeIds", "inputNodeIds"] {
                    if let Some(source_ids) = values.get(key).and_then(Value::as_array) {
                        for source_id in source_ids.iter().filter_map(Value::as_str) {
                            push(source_id, ids, seen);
                        }
                    }
                }
                for value in values.values() {
                    if value.is_object() || value.is_array() {
                        visit(value, ids, seen);
                    }
                }
            }
            _ => {}
        }
    }

    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    visit(value, &mut ids, &mut seen);
    if ids.len() > 64 {
        return Err("action-input-invalid: at most 64 source resources are supported".into());
    }
    Ok(ids)
}

use std::collections::HashSet;

use serde_json::Value;

pub(crate) fn source_node_ids(value: &Value) -> Result<Vec<String>, String> {
    fn push(node_id: &str, node_ids: &mut Vec<String>, seen: &mut HashSet<String>) {
        if seen.insert(node_id.to_owned()) {
            node_ids.push(node_id.to_owned());
        }
    }

    fn visit(value: &Value, node_ids: &mut Vec<String>, seen: &mut HashSet<String>) {
        match value {
            Value::Array(values) => values.iter().for_each(|value| visit(value, node_ids, seen)),
            Value::Object(values) => {
                for key in ["nodeId", "sourceNodeId"] {
                    if let Some(id) = values.get(key).and_then(Value::as_str) {
                        push(id, node_ids, seen);
                    }
                }
                for key in ["nodeIds", "inputNodeIds"] {
                    if let Some(source_ids) = values.get(key).and_then(Value::as_array) {
                        for source_id in source_ids.iter().filter_map(Value::as_str) {
                            push(source_id, node_ids, seen);
                        }
                    }
                }
                for value in values.values() {
                    if value.is_object() || value.is_array() {
                        visit(value, node_ids, seen);
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

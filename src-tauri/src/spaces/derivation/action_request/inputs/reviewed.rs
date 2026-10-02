use std::collections::HashMap;

use serde_json::Value;

pub(super) fn reviewed_revisions(snapshot: &Value) -> Result<HashMap<String, String>, String> {
    fn visit(value: &Value, revisions: &mut HashMap<String, String>) -> Result<(), String> {
        match value {
            Value::Object(fields) => {
                if let Some(approved) = fields.get("approvedRevision").and_then(Value::as_str) {
                    let node_id = fields
                        .get("nodeId")
                        .and_then(Value::as_str)
                        .ok_or("action-input-invalid: reviewed resource is missing")?;
                    if revisions.insert(node_id.into(), approved.into()).is_some() {
                        return Err(
                            "action-input-invalid: resource has multiple review decisions".into(),
                        );
                    }
                }
                for nested in fields.values() {
                    visit(nested, revisions)?;
                }
            }
            Value::Array(values) => {
                for nested in values {
                    visit(nested, revisions)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let mut revisions = HashMap::new();
    visit(snapshot, &mut revisions)?;
    Ok(revisions)
}

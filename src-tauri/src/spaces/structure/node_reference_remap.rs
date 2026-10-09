use std::collections::HashMap;

use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn load_object_fields(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<HashMap<String, Value>, String> {
    let rows = sqlx::query(
        "SELECT id,fields FROM spaces_object_types WHERE space_id=$1 OR space_id IS NULL",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            let id: String = row.try_get("id").map_err(db::sql_error)?;
            let fields: String = row.try_get("fields").map_err(db::sql_error)?;
            Ok((id, serde_json::from_str(&fields).map_err(db::json_error)?))
        })
        .collect()
}

pub(super) fn remap(
    payload: &mut Value,
    node_id_by_source_id: &HashMap<String, String>,
    structure_node_id_by_source_id: &HashMap<String, String>,
    object_fields: Option<&Value>,
) -> Result<(), String> {
    match payload.get("kind").and_then(Value::as_str) {
        Some("portal") => {
            replace_field(payload, "targetNodeId", node_id_by_source_id);
            replace_field(
                payload,
                "targetStructureNodeId",
                structure_node_id_by_source_id,
            );
        }
        Some("object") => {
            if let Some(fields) = object_fields {
                let fields = fields
                    .as_array()
                    .ok_or("Knowledge Type fields are invalid")?;
                for field in fields {
                    if field.get("kind").and_then(Value::as_str) == Some("node-ref") {
                        let key = field
                            .get("key")
                            .and_then(Value::as_str)
                            .ok_or("Knowledge Type node-ref field key is invalid")?;
                        if let Some(properties) = payload.get_mut("properties") {
                            replace_field(properties, key, node_id_by_source_id);
                        }
                    }
                }
            }
            crate::spaces::knowledge::model::revision::apply(payload)?;
        }
        _ => {}
    }
    Ok(())
}

fn replace_field(value: &mut Value, field: &str, node_id_by_source_id: &HashMap<String, String>) {
    if let Some(current) = value.get(field).and_then(Value::as_str) {
        if let Some(replacement) = node_id_by_source_id.get(current) {
            value[field] = Value::String(replacement.clone());
        }
    }
}

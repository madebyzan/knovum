use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use super::scalars::{is_javascript_safe_number, optional_string, required_string};
use crate::spaces::db;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FieldDefinition {
    key: String,
    kind: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    options: Vec<String>,
}

pub(super) async fn validate_object(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    payload: &Value,
) -> Result<String, String> {
    let object = payload
        .as_object()
        .ok_or_else(|| "Knowledge object data must be an object".to_string())?;
    let object_type_id = required_string(object, "objectTypeId", 128)?;
    let status = required_string(object, "status", 32)?;
    optional_string(object, "summary", 16_000, true)?;
    validate_object_status(status)?;
    validate_system_object_status(object_type_id, status)?;
    let row = sqlx::query(
        "SELECT fields FROM spaces_object_types WHERE id=$1 AND (system=1 OR space_id=$2)",
    )
    .bind(object_type_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| "Knowledge object type does not exist in this Space".to_string())?;
    let fields_raw: String = row.try_get("fields").map_err(db::sql_error)?;
    let fields: Vec<FieldDefinition> = serde_json::from_str(&fields_raw).map_err(db::json_error)?;
    validate_properties(tx, space_id, payload, &fields).await?;
    Ok(object_type_id.to_owned())
}

fn validate_system_object_status(object_type_id: &str, status: &str) -> Result<(), String> {
    let valid = match object_type_id {
        "system:question" => matches!(status, "open" | "resolved" | "invalidated"),
        "system:claim" | "system:evidence" | "system:concept" => matches!(
            status,
            "draft" | "accepted" | "contested" | "superseded" | "invalidated"
        ),
        "system:decision" => matches!(status, "draft" | "accepted" | "superseded" | "invalidated"),
        "system:experiment" => matches!(status, "draft" | "accepted" | "invalidated"),
        _ => true,
    };
    valid.then_some(()).ok_or_else(|| {
        format!("Status '{status}' is not valid for Knowledge Type '{object_type_id}'")
    })
}

async fn validate_properties(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    payload: &Value,
    fields: &[FieldDefinition],
) -> Result<(), String> {
    let properties = payload
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "Knowledge object properties must be an object".to_string())?;
    let definitions: HashMap<&str, &FieldDefinition> = fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect();
    for key in properties.keys() {
        if !definitions.contains_key(key.as_str()) {
            return Err(format!(
                "Knowledge object property '{key}' is not declared by its type"
            ));
        }
    }

    let mut node_refs = HashSet::new();
    for field in fields {
        let value = properties.get(&field.key);
        if field.required && value.is_none_or(Value::is_null) {
            return Err(format!(
                "Knowledge object property '{}' is required",
                field.key
            ));
        }
        if let Some(value) = value.filter(|value| !value.is_null()) {
            validate_field_shape(field, value, &mut node_refs)?;
        }
    }
    validate_node_refs(tx, space_id, node_refs).await
}

fn validate_field_shape(
    field: &FieldDefinition,
    value: &Value,
    node_refs: &mut HashSet<String>,
) -> Result<(), String> {
    let valid = match field.kind.as_str() {
        "text" | "date" => value.is_string(),
        "number" => is_javascript_safe_number(value),
        "boolean" => value.is_boolean(),
        "enum" => value.as_str().is_some_and(|selected_option| {
            field.options.iter().any(|option| option == selected_option)
        }),
        "node-ref" => {
            let id = value
                .as_str()
                .filter(|id| !id.is_empty())
                .ok_or_else(|| format!("Property '{}' must be a node id", field.key))?;
            node_refs.insert(id.to_owned());
            true
        }
        _ => {
            return Err(format!(
                "Object type field '{}' has unsupported kind",
                field.key
            ));
        }
    };
    valid.then_some(()).ok_or_else(|| {
        format!(
            "Knowledge object property '{}' has the wrong value type",
            field.key
        )
    })
}

async fn validate_node_refs(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_refs: HashSet<String>,
) -> Result<(), String> {
    if node_refs.is_empty() {
        return Ok(());
    }
    let refs = node_refs.into_iter().collect::<Vec<_>>();
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL \
         AND kind NOT IN ('credential','portal') AND id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(&refs))
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if count != refs.len() as i64 {
        return Err(
            "Knowledge object node-ref must target semantic nodes in the same Space".into(),
        );
    }
    Ok(())
}

pub(crate) fn validate_object_status(status: &str) -> Result<(), String> {
    match status {
        "draft" | "accepted" | "contested" | "superseded" | "invalidated" | "open" | "resolved" => {
            Ok(())
        }
        _ => Err("Unsupported knowledge object status".into()),
    }
}

use std::collections::HashSet;

use serde_json::Value;
use sqlx::Row;

use crate::spaces::{db, knowledge::object_references::collect_node_ids};

pub(super) async fn validate(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    selected: &HashSet<&str>,
) -> Result<(), String> {
    reject_unsettled_outputs(tx, space_id, selected).await?;
    let rows = sqlx::query(
        "SELECT n.id,n.data,n.object_type_id,t.fields FROM spaces_nodes n \
         LEFT JOIN spaces_object_types t ON t.id=n.object_type_id \
         WHERE n.space_id=$1 AND n.deleted_at IS NULL AND n.kind IN ('object')",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        validate_row(row, selected)?;
    }
    Ok(())
}

/// A derivation still running settles its output into the Space it was reserved in, so that
/// output stays until it settles. A settled derivation follows its output.
async fn reject_unsettled_outputs(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    selected: &HashSet<&str>,
) -> Result<(), String> {
    let rows = sqlx::query(
        "SELECT n.data FROM spaces_knowledge_derivations d \
         JOIN spaces_nodes n ON n.id = d.output_node_id \
         WHERE d.space_id = $1 AND d.status = 'running' \
         AND d.output_node_id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(selected))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;

    if let Some(row) = rows.first() {
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        let title = data
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Untitled");
        return Err(format!(
            "Node “{title}” is still being produced and can move between Spaces once it settles"
        ));
    }
    Ok(())
}

fn validate_row(row: sqlx::sqlite::SqliteRow, selected: &HashSet<&str>) -> Result<(), String> {
    let id: String = row.try_get("id").map_err(db::sql_error)?;
    let raw: String = row.try_get("data").map_err(db::sql_error)?;
    let data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
    let is_selected = selected.contains(id.as_str());
    validate_object(&row, &data, is_selected, selected)
}

fn validate_object(
    row: &sqlx::sqlite::SqliteRow,
    payload: &Value,
    is_selected: bool,
    selected: &HashSet<&str>,
) -> Result<(), String> {
    if is_selected
        && row
            .try_get::<Option<String>, _>("object_type_id")
            .map_err(db::sql_error)?
            .is_some_and(|type_id| !type_id.starts_with("system:"))
    {
        return Err(
            "Knowledge Objects using a Space-local custom Type cannot move between Spaces".into(),
        );
    }
    let fields_raw: Option<String> = row.try_get("fields").map_err(db::sql_error)?;
    for target in collect_node_ids(fields_raw.as_deref(), payload)? {
        if is_selected != selected.contains(target.as_str()) {
            return Err(
                "Knowledge object node-ref dependencies must move together between Spaces".into(),
            );
        }
    }
    Ok(())
}

use std::collections::HashSet;

use serde_json::Value;
use sqlx::Row;

use crate::spaces::db;
use crate::spaces::knowledge::object_references::collect_node_ids;

pub(super) async fn active_unique_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    requested: &[String],
    deleting: bool,
) -> Result<Vec<String>, String> {
    let mut unique = Vec::new();
    let mut seen = HashSet::new();
    for id in requested {
        if id.trim().is_empty() {
            return Err("Space node ID cannot be empty".into());
        }
        if seen.insert(id.as_str()) {
            unique.push(id.clone());
        }
    }
    let state_predicate = if deleting {
        "deleted_at IS NULL"
    } else {
        "deleted_at IS NOT NULL"
    };
    let mut sql = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT id FROM spaces_nodes WHERE space_id=$1 AND ",
    );
    sql.push(state_predicate)
        .push(" AND id IN (SELECT value FROM json_each($2))");
    let found_rows = sql
        .build_query_scalar::<String>()
        .bind(space_id)
        .bind(sqlx::types::Json(&unique))
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    let found: HashSet<String> = found_rows.into_iter().collect();
    if found.len() != unique.len() {
        let missing = unique
            .iter()
            .filter(|id| !found.contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        let action = if deleting { "move to Trash" } else { "restore" };
        return Err(format!(
            "Cannot {action} because these IDs are not in the required state for this Space: {}",
            missing.join(", ")
        ));
    }
    Ok(unique)
}

pub(super) async fn reject_live_dependents(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_ids: &[String],
) -> Result<(), String> {
    let selected: HashSet<&str> = node_ids.iter().map(String::as_str).collect();
    let rows = sqlx::query(
        "SELECT n.id,n.kind,n.data,t.fields FROM spaces_nodes n \
             LEFT JOIN spaces_object_types t ON t.id=n.object_type_id \
             WHERE n.space_id=$1 AND n.deleted_at IS NULL AND n.kind IN ('portal','object')",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        if selected.contains(id.as_str()) {
            continue;
        }
        let kind: String = row.try_get("kind").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        let dependent = match kind.as_str() {
            "portal" => {
                data.get("targetSpaceId").and_then(Value::as_str) == Some(space_id)
                    && data
                        .get("targetNodeId")
                        .and_then(Value::as_str)
                        .is_some_and(|target| selected.contains(target))
            }
            "object" => {
                let fields_raw: Option<String> = row.try_get("fields").map_err(db::sql_error)?;
                collect_node_ids(fields_raw.as_deref(), &data)?
                    .iter()
                    .any(|target| selected.contains(target.as_str()))
            }
            _ => false,
        };
        if dependent {
            return Err(format!(
                "Cannot move node to Trash while active {kind} node '{id}' still references it"
            ));
        }
    }
    Ok(())
}

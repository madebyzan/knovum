use std::collections::{HashMap, HashSet};

use serde_json::Value;
use sqlx::Row;

use super::contracts::DeletedNodeSelection;
use crate::spaces::db;
use crate::spaces::knowledge::object_references::collect_node_ids;

pub(super) async fn deletable_nodes(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &DeletedNodeSelection,
) -> Result<Vec<(String, String)>, String> {
    let unique = super::super::dedup_and_validate_node_ids(
        &request.ids,
        "Space delete item ID cannot be empty",
    )?;
    let requested_json = serde_json::to_string(&unique).map_err(db::json_error)?;
    let rows = sqlx::query(
        "SELECT id,data FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NOT NULL \
             AND id IN (SELECT value FROM json_each($2))",
    )
    .bind(&request.space_id)
    .bind(requested_json)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut by_id = HashMap::new();
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let data: String = row.try_get("data").map_err(db::sql_error)?;
        by_id.insert(id, data);
    }
    if by_id.len() != unique.len() {
        let missing = unique
            .iter()
            .filter(|id| !by_id.contains_key(*id))
            .cloned()
            .collect::<Vec<_>>();
        return Err(format!(
            "Cannot permanently delete because these IDs are not trashed nodes in this Space: {}",
            missing.join(", ")
        ));
    }
    unique
        .into_iter()
        .map(|id| {
            by_id
                .remove(&id)
                .map(|data| (id.clone(), data))
                .ok_or_else(|| format!("Validated delete selection disappeared: {id}"))
        })
        .collect()
}

pub(super) async fn reject_persistent_dependents(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    erased: &HashSet<String>,
) -> Result<(), String> {
    let rows = sqlx::query(
        "SELECT n.id,n.space_id,n.kind,n.data,t.fields FROM spaces_nodes n \
             LEFT JOIN spaces_object_types t ON t.id=n.object_type_id \
             WHERE n.kind IN ('portal','object')",
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        if erased.contains(&id) {
            continue;
        }
        let node_space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
        let kind: String = row.try_get("kind").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        let references_erased = match kind.as_str() {
            "portal" => {
                data.get("targetSpaceId").and_then(Value::as_str) == Some(space_id)
                    && data
                        .get("targetNodeId")
                        .and_then(Value::as_str)
                        .is_some_and(|target| erased.contains(target))
            }
            "object" if node_space_id == space_id => {
                let fields_raw: Option<String> = row.try_get("fields").map_err(db::sql_error)?;
                collect_node_ids(fields_raw.as_deref(), &data)?
                    .iter()
                    .any(|target| erased.contains(target))
            }
            _ => false,
        };
        if references_erased {
            return Err(format!(
                "Cannot permanently delete node while retained {kind} node '{id}' still references it"
            ));
        }
    }
    Ok(())
}

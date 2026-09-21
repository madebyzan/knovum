use std::collections::HashSet;

use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(crate) async fn validate_structure_targets(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), String> {
    let rows =
        sqlx::query("SELECT id,data FROM spaces_nodes WHERE kind='portal' AND deleted_at IS NULL")
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    for row in rows {
        let portal_id: String = row.try_get("id").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        let Some(structure_node_id) = data.get("targetStructureNodeId").and_then(Value::as_str)
        else {
            continue;
        };
        let space_id = data
            .get("targetSpaceId")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("Portal {portal_id} has no target Space"))?;
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM spaces_structure_nodes \
             WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL \
             AND kind IN ('group','section')",
        )
        .bind(structure_node_id)
        .bind(space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if count != 1 {
            return Err(format!(
                "Portal {portal_id} targets a missing structure container"
            ));
        }
    }
    Ok(())
}

pub(crate) async fn reject_structure_targets(
    tx: &mut Transaction<'_, Sqlite>,
    target_space_id: &str,
    target_structure_node_ids: &HashSet<&str>,
) -> Result<(), String> {
    if target_structure_node_ids.is_empty() {
        return Ok(());
    }
    let rows =
        sqlx::query("SELECT id,data FROM spaces_nodes WHERE kind='portal' AND deleted_at IS NULL")
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    for row in rows {
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        let targets_space =
            data.get("targetSpaceId").and_then(Value::as_str) == Some(target_space_id);
        let targets_structure = data
            .get("targetStructureNodeId")
            .and_then(Value::as_str)
            .is_some_and(|id| target_structure_node_ids.contains(id));
        if targets_space && targets_structure {
            let portal_id: String = row.try_get("id").map_err(db::sql_error)?;
            return Err(format!(
                "Portal {portal_id} must be removed before permanently deleting its target container"
            ));
        }
    }
    Ok(())
}

pub(crate) async fn reject_node_targets(
    tx: &mut Transaction<'_, Sqlite>,
    target_space_id: &str,
    target_node_ids: &HashSet<&str>,
) -> Result<(), String> {
    if target_node_ids.is_empty() {
        return Ok(());
    }
    let rows =
        sqlx::query("SELECT id,data FROM spaces_nodes WHERE kind='portal' AND deleted_at IS NULL")
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    for row in rows {
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        let targets_space =
            data.get("targetSpaceId").and_then(Value::as_str) == Some(target_space_id);
        let targets_node = data
            .get("targetNodeId")
            .and_then(Value::as_str)
            .is_some_and(|id| target_node_ids.contains(id));
        if targets_space && targets_node {
            let portal_id: String = row.try_get("id").map_err(db::sql_error)?;
            return Err(format!(
                "Portal {portal_id} must be removed before discarding its target content"
            ));
        }
    }
    Ok(())
}

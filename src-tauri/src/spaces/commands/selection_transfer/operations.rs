use std::collections::HashSet;

use serde_json::Value;
use sqlx::{types::Json, Row};

use super::contracts::TransferSelectionRequest;
use super::dependencies::Dependencies;
use crate::spaces::db;

pub(super) async fn update_portals(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &TransferSelectionRequest,
    selected: &HashSet<&str>,
    selected_structure_nodes: &HashSet<&str>,
    now: i64,
) -> Result<(), String> {
    let rows =
        sqlx::query("SELECT id,data FROM spaces_nodes WHERE deleted_at IS NULL AND kind='portal'")
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let mut data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        if data.get("targetSpaceId").and_then(Value::as_str)
            != Some(request.source_space_id.as_str())
        {
            continue;
        }
        let follows_moved_node = data
            .get("targetNodeId")
            .and_then(Value::as_str)
            .is_some_and(|target| selected.contains(target));
        let follows_moved_structure_node = data
            .get("targetStructureNodeId")
            .and_then(Value::as_str)
            .is_some_and(|target| selected_structure_nodes.contains(target));
        if !follows_moved_node && !follows_moved_structure_node {
            continue;
        }
        if data.get("targetSceneId").is_some() {
            return Err("A Portal targeting both a Saved View and a moved node must be edited before moving that node between Spaces".into());
        }
        data.as_object_mut()
            .ok_or_else(|| "Portal data is invalid".to_string())?
            .insert(
                "targetSpaceId".into(),
                Value::String(request.target_space_id.clone()),
            );
        sqlx::query("UPDATE spaces_nodes SET data=$1,updated_at=$2 WHERE id=$3")
            .bind(sqlx::types::Json(&data))
            .bind(now)
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

/// Moves the dependencies whose ends all moved. Call once their nodes are in the target Space:
/// triggers hold every relation and provenance row to its endpoints' Space.
pub(super) async fn move_dependencies(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &TransferSelectionRequest,
    dependencies: &Dependencies,
    now: i64,
) -> Result<(), String> {
    // A relation's style is keyed by its Space; the target snapshot writes it back.
    sqlx::query(
        "DELETE FROM spaces_relation_styles \
         WHERE space_id=$1 AND relation_id IN (SELECT value FROM json_each($2))",
    )
    .bind(&request.source_space_id)
    .bind(Json(&dependencies.relations.moved))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for (statement, ids) in [
        (
            "UPDATE spaces_relations SET space_id=$1,updated_at=$2 \
             WHERE space_id=$3 AND id IN (SELECT value FROM json_each($4))",
            &dependencies.relations.moved,
        ),
        (
            "UPDATE spaces_provenance SET space_id=$1,updated_at=$2 \
             WHERE space_id=$3 AND id IN (SELECT value FROM json_each($4))",
            &dependencies.provenance.moved,
        ),
    ] {
        sqlx::query(statement)
            .bind(&request.target_space_id)
            .bind(now)
            .bind(&request.source_space_id)
            .bind(Json(ids))
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn remove_scene_focus(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    selected: &HashSet<&str>,
    now: i64,
) -> Result<(), String> {
    let rows =
        sqlx::query("SELECT id,focus_structure_node_ids FROM spaces_scenes WHERE space_id=$1")
            .bind(space_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let raw: String = row
            .try_get("focus_structure_node_ids")
            .map_err(db::sql_error)?;
        let mut focus: Vec<String> = serde_json::from_str(&raw).map_err(db::json_error)?;
        let before = focus.len();
        focus.retain(|structure_node_id| !selected.contains(structure_node_id.as_str()));
        if before == focus.len() {
            continue;
        }
        sqlx::query(
            "UPDATE spaces_scenes SET focus_structure_node_ids=$1,updated_at=$2 WHERE id=$3",
        )
        .bind(sqlx::types::Json(&focus))
        .bind(now)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}

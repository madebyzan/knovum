use std::collections::HashSet;

use serde_json::Value;
use sqlx::Row;

use super::contracts::TransferSelectionRequest;
use crate::spaces::db;

pub(super) async fn moved_relation_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    selected: &HashSet<&str>,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query("SELECT id,source_id,target_id FROM spaces_relations WHERE space_id=$1")
        .bind(space_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    let mut ids = Vec::new();
    for row in rows {
        let source: String = row.try_get("source_id").map_err(db::sql_error)?;
        let target: String = row.try_get("target_id").map_err(db::sql_error)?;
        if selected.contains(source.as_str()) && selected.contains(target.as_str()) {
            ids.push(row.try_get("id").map_err(db::sql_error)?);
        }
    }
    Ok(ids)
}

pub(super) async fn moved_provenance_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    selected: &HashSet<&str>,
) -> Result<Vec<String>, String> {
    let rows = sqlx::query("SELECT id,source_node_id FROM spaces_provenance WHERE space_id=$1")
        .bind(space_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    let mut ids = Vec::new();
    for row in rows {
        let source: String = row.try_get("source_node_id").map_err(db::sql_error)?;
        if selected.contains(source.as_str()) {
            ids.push(row.try_get("id").map_err(db::sql_error)?);
        }
    }
    Ok(ids)
}

pub(super) async fn update_portals(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &TransferSelectionRequest,
    selected: &HashSet<&str>,
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
        if !follows_moved_node {
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
        let serialized = serde_json::to_string(&data).map_err(db::json_error)?;
        sqlx::query("UPDATE spaces_nodes SET data=$1,updated_at=$2 WHERE id=$3")
            .bind(serialized)
            .bind(now)
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn move_provenance(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &TransferSelectionRequest,
    selected: &HashSet<&str>,
    now: i64,
) -> Result<(), String> {
    let rows = sqlx::query(
        "SELECT p.id,p.source_node_id,p.target_kind,p.target_id,r.source_id,r.target_id FROM spaces_provenance p \
         LEFT JOIN spaces_relations r ON p.target_kind='relation' AND r.id=p.target_id WHERE p.space_id=$1",
    ).bind(&request.source_space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    for row in rows {
        let source: String = row.try_get("source_node_id").map_err(db::sql_error)?;
        if !selected.contains(source.as_str()) {
            continue;
        }
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        sqlx::query("UPDATE spaces_provenance SET space_id=$1,updated_at=$2 WHERE id=$3")
            .bind(&request.target_space_id)
            .bind(now)
            .bind(id)
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
    let rows = sqlx::query("SELECT id,focus_node_ids FROM spaces_scenes WHERE space_id=$1")
        .bind(space_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let raw: String = row.try_get("focus_node_ids").map_err(db::sql_error)?;
        let mut focus: Vec<String> = serde_json::from_str(&raw).map_err(db::json_error)?;
        let before = focus.len();
        focus.retain(|node_id| !selected.contains(node_id.as_str()));
        if before == focus.len() {
            continue;
        }
        sqlx::query("UPDATE spaces_scenes SET focus_node_ids=$1,updated_at=$2 WHERE id=$3")
            .bind(serde_json::to_string(&focus).map_err(db::json_error)?)
            .bind(now)
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

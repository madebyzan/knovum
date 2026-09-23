use std::collections::HashSet;

use sqlx::{Row, Sqlite, Transaction};

use super::{apply_relations, group, RelationLifecycle};
use crate::spaces::{
    db,
    structure::{portal_integrity, repository, repository::SpaceStructureSnapshot},
};
pub(in crate::spaces::structure) async fn apply(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    current_revision: i64,
    current_snapshot: &SpaceStructureSnapshot,
    redo: bool,
) -> Result<i64, String> {
    let order = if redo { "ASC" } else { "DESC" };
    let undone = if redo { 1 } else { 0 };
    let sql = format!("SELECT id,before_snapshot,after_snapshot,before_node_ids,after_node_ids,before_relations,after_relations,deletion_batch_id,deletion_batch_restored_after,history_group_id FROM spaces_edit_history WHERE space_id=$1 AND undone=$2 ORDER BY id {order} LIMIT 1");
    let row = sqlx::query(&sql)
        .bind(space_id)
        .bind(undone)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?
        .ok_or_else(|| {
            if redo {
                "Nothing to redo".to_string()
            } else {
                "Nothing to undo".to_string()
            }
        })?;
    let id: i64 = row.try_get("id").map_err(db::sql_error)?;
    let group_id: Option<String> = row.try_get("history_group_id").map_err(db::sql_error)?;
    if let Some(group_id) = group_id {
        return group::apply(tx, &group_id, space_id, current_revision, redo).await;
    }
    let before_snapshot_json: String = row.try_get("before_snapshot").map_err(db::sql_error)?;
    let after_snapshot_json: String = row.try_get("after_snapshot").map_err(db::sql_error)?;
    let node_ids_json: String = if redo {
        row.try_get("after_node_ids")
    } else {
        row.try_get("before_node_ids")
    }
    .map_err(db::sql_error)?;
    let relations_json: String = if redo {
        row.try_get("after_relations")
    } else {
        row.try_get("before_relations")
    }
    .map_err(db::sql_error)?;
    let before_snapshot: SpaceStructureSnapshot =
        serde_json::from_str(&before_snapshot_json).map_err(db::json_error)?;
    let after_snapshot: SpaceStructureSnapshot =
        serde_json::from_str(&after_snapshot_json).map_err(db::json_error)?;
    let mut snapshot = if redo {
        after_snapshot.clone()
    } else {
        before_snapshot.clone()
    };
    let current_heights = super::neutral::content_height_states(std::iter::once(current_snapshot));
    let before_heights = super::neutral::content_height_states(std::iter::once(&before_snapshot));
    let after_heights = super::neutral::content_height_states(std::iter::once(&after_snapshot));
    super::neutral::preserve_content_heights(
        &mut snapshot,
        &current_heights,
        &before_heights,
        &after_heights,
    );
    let node_ids: Vec<String> = serde_json::from_str(&node_ids_json).map_err(db::json_error)?;
    let relations: Vec<RelationLifecycle> =
        serde_json::from_str(&relations_json).map_err(db::json_error)?;
    apply_nodes(tx, space_id, &node_ids).await?;
    apply_relations(tx, &relations).await?;
    let next = repository::save_in_transaction(tx, space_id, current_revision, &snapshot).await?;
    portal_integrity::validate_structure_targets(tx).await?;
    let deletion_batch_id: Option<String> =
        row.try_get("deletion_batch_id").map_err(db::sql_error)?;
    if let Some(batch_id) = deletion_batch_id {
        let restored_after: bool = row
            .try_get::<Option<bool>, _>("deletion_batch_restored_after")
            .map_err(db::sql_error)?
            .unwrap_or(false);
        let should_restore = if redo {
            restored_after
        } else {
            !restored_after
        };
        sqlx::query("UPDATE spaces_deleted_structure_batches SET restored_at=$1 WHERE id=$2 AND space_id=$3")
            .bind(if should_restore { Some(crate::clock::unix_time_millis()?) } else { None })
            .bind(batch_id).bind(space_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    sqlx::query(
        "UPDATE spaces_edit_history SET undone=$1,expected_revision=$2 WHERE id=$3 AND space_id=$4",
    )
    .bind(if redo { 0 } else { 1 })
    .bind(next)
    .bind(id)
    .bind(space_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(next)
}

async fn apply_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    target: &[String],
) -> Result<(), String> {
    let current: Vec<String> =
        sqlx::query_scalar("SELECT id FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL")
            .bind(space_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    let current_set: HashSet<&str> = current.iter().map(String::as_str).collect();
    let target_set: HashSet<&str> = target.iter().map(String::as_str).collect();
    let trash = current
        .iter()
        .filter(|id| !target_set.contains(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let restore = target
        .iter()
        .filter(|id| !current_set.contains(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let now = crate::clock::unix_time_millis()?;
    if !trash.is_empty() {
        crate::spaces::nodes::trash_in_transaction(tx, space_id, &trash, now).await?;
    }
    if !restore.is_empty() {
        crate::spaces::nodes::restore_in_transaction(tx, space_id, &restore, now).await?;
    }
    Ok(())
}

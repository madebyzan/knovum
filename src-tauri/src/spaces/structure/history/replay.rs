use crate::foundation::app_error::AppError;
use std::collections::HashSet;

use sqlx::{Row, Sqlite, Transaction};

use super::{apply_relations, group, RelationLifecycle};
use crate::spaces::{
    db,
    structure::{portal_integrity, repository, repository::SpaceStructureSnapshot},
};
pub(in crate::spaces) async fn apply(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    current_revision: i64,
    current_snapshot: &SpaceStructureSnapshot,
    redo: bool,
) -> Result<i64, AppError> {
    let order = if redo { "ASC" } else { "DESC" };
    let undone = if redo { 1 } else { 0 };
    let mut sql = sqlx::QueryBuilder::<Sqlite>::new("SELECT id,before_snapshot,after_snapshot,before_node_ids,after_node_ids,before_relations,after_relations,history_group_id FROM spaces_edit_history WHERE space_id=$1 AND undone=$2 ORDER BY id ");
    sql.push(order).push(" LIMIT 1");
    let row = sql
        .build()
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
    let current_sizes = super::neutral::content_size_states(std::iter::once(current_snapshot));
    let before_sizes = super::neutral::content_size_states(std::iter::once(&before_snapshot));
    let after_sizes = super::neutral::content_size_states(std::iter::once(&after_snapshot));
    super::neutral::preserve_content_sizes(
        &mut snapshot,
        &current_sizes,
        &before_sizes,
        &after_sizes,
    );
    let node_ids: Vec<String> = serde_json::from_str(&node_ids_json).map_err(db::json_error)?;
    let relations: Vec<RelationLifecycle> =
        serde_json::from_str(&relations_json).map_err(db::json_error)?;
    apply_nodes(tx, space_id, &node_ids).await?;
    apply_relations(tx, &relations).await?;
    crate::spaces::nodes::fit_content_sizes_in_snapshot(tx, space_id, &mut snapshot, &node_ids)
        .await?;
    let next = repository::save_in_transaction(tx, space_id, current_revision, &snapshot).await?;
    portal_integrity::validate_structure_targets(tx, &[space_id]).await?;
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
) -> Result<(), AppError> {
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
    let now = crate::foundation::clock::unix_time_millis()?;
    if !trash.is_empty() {
        crate::spaces::nodes::trash_in_transaction(tx, space_id, &trash, now).await?;
    }
    if !restore.is_empty() {
        crate::spaces::nodes::restore_in_transaction(tx, space_id, &restore, now).await?;
    }
    Ok(())
}

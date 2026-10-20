use std::collections::HashSet;

use crate::foundation::app_error::AppError;
use sqlx::{Row, Sqlite, Transaction};

use super::patch::{HistoryPatch, Side};
use super::{apply_relation_styles, apply_relations, apply_structure, fit_restored_content, group};
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
    let (id, group_id, patch) = top(tx, space_id, redo).await?;
    if let Some(group_id) = group_id {
        return group::apply(tx, &group_id, space_id, current_revision, redo).await;
    }
    let side = Side::of(redo);
    apply_nodes(tx, space_id, &patch, side).await?;
    apply_relations(tx, &patch, side).await?;
    repository::rows::vacate(tx, space_id, &patch.structure_ids()).await?;
    apply_structure(tx, space_id, &patch, side, current_snapshot).await?;
    apply_relation_styles(tx, space_id, &patch, side).await?;
    let (mut snapshot, _) = repository::load_in_transaction(tx, space_id).await?;
    fit_restored_content(tx, space_id, &mut snapshot, &patch.present_nodes(side)).await?;
    portal_integrity::validate_structure_targets(tx, &[space_id]).await?;
    let next = repository::advance_revision(tx, space_id, current_revision).await?;
    settle(tx, id, redo).await?;
    Ok(next)
}

/// The record a Space would undo or redo next. Undo takes the newest applied edit, redo the oldest
/// undone one, so the two walk the same stack from opposite ends.
pub(super) async fn top(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    redo: bool,
) -> Result<(i64, Option<String>, HistoryPatch), AppError> {
    let sql = if redo {
        "SELECT id,history_group_id,patch FROM spaces_edit_history \
         WHERE space_id=$1 AND undone=1 ORDER BY id ASC LIMIT 1"
    } else {
        "SELECT id,history_group_id,patch FROM spaces_edit_history \
         WHERE space_id=$1 AND undone=0 ORDER BY id DESC LIMIT 1"
    };
    let row = sqlx::query(sql)
        .bind(space_id)
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
    let encoded: String = row.try_get("patch").map_err(db::sql_error)?;
    Ok((
        row.try_get("id").map_err(db::sql_error)?,
        row.try_get("history_group_id").map_err(db::sql_error)?,
        serde_json::from_str(&encoded).map_err(db::json_error)?,
    ))
}

pub(super) async fn settle(
    tx: &mut Transaction<'_, Sqlite>,
    id: i64,
    redo: bool,
) -> Result<(), String> {
    sqlx::query("UPDATE spaces_edit_history SET undone=$1 WHERE id=$2")
        .bind(i64::from(!redo))
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

/// Content leaves and returns through the Trash, so a replay only has to move the nodes whose
/// presence the record changed; the node rows themselves are never rewritten.
async fn apply_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    patch: &HistoryPatch,
    side: Side,
) -> Result<(), AppError> {
    let touched = patch.touched_nodes();
    if touched.is_empty() {
        return Ok(());
    }
    let expected: HashSet<String> = patch.present_nodes(side).into_iter().collect();
    let live: HashSet<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL \
         AND id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(&touched))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .into_iter()
    .collect();
    let trash = touched
        .iter()
        .filter(|id| live.contains(*id) && !expected.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    let restore = touched
        .iter()
        .filter(|id| !live.contains(*id) && expected.contains(*id))
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

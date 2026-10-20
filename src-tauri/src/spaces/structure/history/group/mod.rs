//! A single edit that spans two Spaces.
//!
//! Moving a subtree between Spaces changes two Spaces at once, so it is remembered as one record
//! per Space under a shared group identity and replayed as one step. Structure rows keep their
//! identity across the move, which is why a replay first takes them out of the Space that is
//! giving them up and only then writes them into the one receiving them.

use crate::foundation::app_error::AppError;
use sqlx::{Row, Sqlite, Transaction};

use super::patch::{self, HistoryPatch, HistoryState, Side};
use super::{insert, HistoryInsert};
use crate::spaces::{db, structure::repository::SpaceStructureSnapshot};

mod portal;
mod replay;

struct TargetState {
    history_id: i64,
    space_id: String,
    revision: i64,
    patch: HistoryPatch,
    current: SpaceStructureSnapshot,
}

pub(super) async fn record(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: &str,
    entries: &[(&str, &HistoryState, &HistoryState)],
) -> Result<(), AppError> {
    let now = crate::foundation::clock::unix_time_millis()?;
    for (space_id, before, after) in entries {
        super::clear_redo(tx, space_id).await?;
        insert(
            tx,
            HistoryInsert {
                space_id,
                patch: &patch::diff(before, after),
                group_id: Some(group_id),
                now,
            },
        )
        .await?;
        super::prune(tx, space_id).await?;
    }
    Ok(())
}

pub(super) async fn apply(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: &str,
    requested_space: &str,
    requested_revision: i64,
    redo: bool,
) -> Result<i64, AppError> {
    let targets = load_targets(tx, group_id, requested_space, requested_revision, redo).await?;
    replay::apply(tx, &targets, requested_space, Side::of(redo), redo).await
}

async fn load_targets(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: &str,
    requested_space: &str,
    requested_revision: i64,
    redo: bool,
) -> Result<Vec<TargetState>, AppError> {
    let rows = sqlx::query(
        "SELECT id,space_id,patch FROM spaces_edit_history \
         WHERE history_group_id=$1 AND undone=$2 ORDER BY id",
    )
    .bind(group_id)
    .bind(i64::from(redo))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if rows.len() != 2 {
        return Err("Coordinated Space history is incomplete".into());
    }
    let mut targets = Vec::new();
    for row in rows {
        let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
        let (current, revision) =
            super::super::repository::load_in_transaction(tx, &space_id).await?;
        if space_id == requested_space && revision != requested_revision {
            return Err(AppError::new(
                "space_edit_revision_conflict",
                "Space edit revision conflict",
            ));
        }
        let history_id: i64 = row.try_get("id").map_err(db::sql_error)?;
        ensure_top(tx, &space_id, history_id, redo).await?;
        let encoded: String = row.try_get("patch").map_err(db::sql_error)?;
        targets.push(TargetState {
            history_id,
            space_id,
            revision,
            patch: serde_json::from_str(&encoded).map_err(db::json_error)?,
            current,
        });
    }
    Ok(targets)
}

/// Neither Space may have moved on since this edit: a coordinated replay has to be the next step
/// on both stacks or it would reorder one Space's history against the other's.
async fn ensure_top(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    history_id: i64,
    redo: bool,
) -> Result<(), AppError> {
    let (top, _, _) = super::replay::top(tx, space_id, redo).await?;
    if top != history_id {
        return Err("Coordinated Space history has diverged".into());
    }
    Ok(())
}

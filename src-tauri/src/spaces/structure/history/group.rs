use sqlx::{Row, Sqlite, Transaction};

use super::{insert, prune, HistoryInsert, HistoryState, RelationLifecycle};
use crate::spaces::{db, structure::repository::SpaceStructureSnapshot};

mod portal;
mod replay;

struct TargetState {
    history_id: i64,
    space_id: String,
    snapshot: SpaceStructureSnapshot,
    before_snapshot: SpaceStructureSnapshot,
    after_snapshot: SpaceStructureSnapshot,
    current_snapshot: SpaceStructureSnapshot,
    node_ids: Vec<String>,
    relations: Vec<RelationLifecycle>,
}

pub(super) async fn record(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: &str,
    entries: &[(&str, &HistoryState, &HistoryState, i64)],
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    for (space_id, before, after, revision) in entries {
        clear_redo(tx, space_id).await?;
        insert(
            tx,
            HistoryInsert {
                space_id,
                before,
                after,
                deletion_batch_id: None,
                deletion_restored_after: None,
                group_id: Some(group_id),
                revision: *revision,
                now,
            },
        )
        .await?;
        prune(tx, space_id).await?;
    }
    Ok(())
}

async fn clear_redo(tx: &mut Transaction<'_, Sqlite>, space_id: &str) -> Result<(), String> {
    let batches: Vec<String> = sqlx::query_scalar("SELECT deletion_batch_id FROM spaces_edit_history WHERE space_id=$1 AND undone=1 AND deletion_batch_id IS NOT NULL").bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    sqlx::query("DELETE FROM spaces_edit_history WHERE space_id=$1 AND undone=1")
        .bind(space_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    for batch in batches {
        sqlx::query("DELETE FROM spaces_deleted_structure_batches WHERE id=$1 AND space_id=$2 AND restored_at IS NOT NULL").bind(batch).bind(space_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn apply(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: &str,
    requested_space: &str,
    requested_revision: i64,
    redo: bool,
) -> Result<i64, String> {
    let mut targets = load_targets(tx, group_id, requested_space, requested_revision, redo).await?;
    replay::preserve_history_neutral_content_heights(&mut targets);
    replay::remove_current_structures(tx, &targets).await?;
    let owners = portal::node_owners(&targets);
    let structure_owners = portal::structure_owners(&targets);
    replay::move_nodes(tx, &owners).await?;
    replay::move_related_data(tx, &targets).await?;
    portal::update_targets(tx, &owners, &structure_owners).await?;
    replay::save_targets(tx, &targets, requested_space, redo).await
}

async fn load_targets(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: &str,
    requested_space: &str,
    requested_revision: i64,
    redo: bool,
) -> Result<Vec<TargetState>, String> {
    let rows = sqlx::query("SELECT id,space_id,before_snapshot,after_snapshot,before_node_ids,after_node_ids,before_relations,after_relations FROM spaces_edit_history WHERE history_group_id=$1 AND undone=$2 ORDER BY id").bind(group_id).bind(if redo {1}else{0}).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    if rows.len() != 2 {
        return Err("Coordinated Space history is incomplete".into());
    }
    let mut targets = Vec::new();
    for row in rows {
        let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
        let (current_snapshot, current) =
            super::super::repository::load_in_transaction(tx, &space_id).await?;
        if space_id == requested_space && current != requested_revision {
            return Err("Space edit revision conflict".into());
        }
        let history_id: i64 = row.try_get("id").map_err(db::sql_error)?;
        ensure_top(tx, &space_id, history_id, redo).await?;
        let before_snapshot_json: String = row.try_get("before_snapshot").map_err(db::sql_error)?;
        let after_snapshot_json: String = row.try_get("after_snapshot").map_err(db::sql_error)?;
        let before_snapshot: SpaceStructureSnapshot =
            serde_json::from_str(&before_snapshot_json).map_err(db::json_error)?;
        let after_snapshot: SpaceStructureSnapshot =
            serde_json::from_str(&after_snapshot_json).map_err(db::json_error)?;
        let snapshot = if redo {
            after_snapshot.clone()
        } else {
            before_snapshot.clone()
        };
        let nodes: String = if redo {
            row.try_get("after_node_ids")
        } else {
            row.try_get("before_node_ids")
        }
        .map_err(db::sql_error)?;
        let relations: String = if redo {
            row.try_get("after_relations")
        } else {
            row.try_get("before_relations")
        }
        .map_err(db::sql_error)?;
        targets.push(TargetState {
            history_id,
            space_id,
            snapshot,
            before_snapshot,
            after_snapshot,
            current_snapshot,
            node_ids: serde_json::from_str(&nodes).map_err(db::json_error)?,
            relations: serde_json::from_str(&relations).map_err(db::json_error)?,
        });
    }
    Ok(targets)
}

async fn current_revision(tx: &mut Transaction<'_, Sqlite>, space_id: &str) -> Result<i64, String> {
    sqlx::query_scalar("SELECT revision FROM spaces_edit_state WHERE space_id=$1")
        .bind(space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)
}

async fn ensure_top(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    history_id: i64,
    redo: bool,
) -> Result<(), String> {
    let sql = if redo {
        "SELECT id FROM spaces_edit_history WHERE space_id=$1 AND undone=1 ORDER BY id ASC LIMIT 1"
    } else {
        "SELECT id FROM spaces_edit_history WHERE space_id=$1 AND undone=0 ORDER BY id DESC LIMIT 1"
    };
    let top: Option<i64> = sqlx::query_scalar(sql)
        .bind(space_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    if top != Some(history_id) {
        return Err("Coordinated Space history has diverged".into());
    }
    Ok(())
}

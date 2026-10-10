use crate::app_error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{Row, Sqlite, Transaction};

use super::repository::SpaceStructureSnapshot;
use crate::spaces::db;

mod group;
mod neutral;
mod replay;

pub(in crate::spaces) use replay::apply;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelationLifecycle {
    id: String,
    status: String,
    status_before_invalidation: Option<String>,
}

pub(crate) struct HistoryState {
    snapshot: SpaceStructureSnapshot,
    node_ids: Vec<String>,
    relations: Vec<RelationLifecycle>,
}

pub(super) struct HistoryInsert<'a> {
    pub space_id: &'a str,
    pub before: &'a HistoryState,
    pub after: &'a HistoryState,
    pub deletion_batch_id: Option<&'a str>,
    pub deletion_restored_after: Option<bool>,
    pub group_id: Option<&'a str>,
    pub revision: i64,
    pub now: i64,
}

pub(crate) async fn capture(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &SpaceStructureSnapshot,
) -> Result<HistoryState, String> {
    let node_ids = sqlx::query_scalar(
        "SELECT id FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL ORDER BY id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let rows = sqlx::query("SELECT id,status,status_before_invalidation FROM spaces_relations WHERE space_id=$1 ORDER BY id")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let relations = rows
        .into_iter()
        .map(|row| {
            Ok(RelationLifecycle {
                id: row.try_get("id").map_err(db::sql_error)?,
                status: row.try_get("status").map_err(db::sql_error)?,
                status_before_invalidation: row
                    .try_get("status_before_invalidation")
                    .map_err(db::sql_error)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(HistoryState {
        snapshot: snapshot.clone(),
        node_ids,
        relations,
    })
}

pub(crate) async fn record(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    before: &HistoryState,
    after: &HistoryState,
    deletion_batch_id: Option<&str>,
    deletion_batch_restored_after: Option<bool>,
    applied_revision: i64,
) -> Result<(), String> {
    clear_redo(tx, space_id).await?;
    let now = crate::clock::unix_time_millis()?;
    insert(
        tx,
        HistoryInsert {
            space_id,
            before,
            after,
            deletion_batch_id,
            deletion_restored_after: deletion_batch_restored_after,
            group_id: None,
            revision: applied_revision,
            now,
        },
    )
    .await?;
    prune(tx, space_id).await
}

pub(super) async fn clear_redo(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(), String> {
    let abandoned_batches: Vec<String> = sqlx::query_scalar("SELECT deletion_batch_id FROM spaces_edit_history WHERE space_id=$1 AND undone=1 AND deletion_batch_id IS NOT NULL")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    sqlx::query("DELETE FROM spaces_edit_history WHERE space_id=$1 AND undone=1")
        .bind(space_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    for batch_id in abandoned_batches {
        sqlx::query("DELETE FROM spaces_deleted_structure_batches WHERE id=$1 AND space_id=$2 AND restored_at IS NOT NULL")
            .bind(batch_id).bind(space_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

pub(crate) async fn record_group(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: &str,
    entries: &[(&str, &HistoryState, &HistoryState, i64)],
) -> Result<(), AppError> {
    group::record(tx, group_id, entries).await
}

async fn insert(tx: &mut Transaction<'_, Sqlite>, entry: HistoryInsert<'_>) -> Result<(), String> {
    let HistoryInsert {
        space_id,
        before,
        after,
        deletion_batch_id,
        deletion_restored_after,
        group_id,
        revision,
        now,
    } = entry;
    sqlx::query("INSERT INTO spaces_edit_history(space_id,before_snapshot,after_snapshot,before_node_ids,after_node_ids,before_relations,after_relations,deletion_batch_id,deletion_batch_restored_after,history_group_id,expected_revision,undone,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,0,$12)")
        .bind(space_id)
        .bind(sqlx::types::Json(&before.snapshot))
        .bind(sqlx::types::Json(&after.snapshot))
        .bind(sqlx::types::Json(&before.node_ids))
        .bind(sqlx::types::Json(&after.node_ids))
        .bind(sqlx::types::Json(&before.relations))
        .bind(sqlx::types::Json(&after.relations))
        .bind(deletion_batch_id).bind(deletion_restored_after).bind(group_id).bind(revision).bind(now)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
async fn prune(tx: &mut Transaction<'_, Sqlite>, space_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM spaces_edit_history WHERE space_id=$1 AND id NOT IN (SELECT id FROM spaces_edit_history WHERE space_id=$1 ORDER BY id DESC LIMIT 100)").bind(space_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}

async fn apply_relations(
    tx: &mut Transaction<'_, Sqlite>,
    relations: &[RelationLifecycle],
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    for relation in relations {
        sqlx::query("UPDATE spaces_relations SET status=$1,status_before_invalidation=$2,updated_at=$3 WHERE id=$4")
            .bind(&relation.status).bind(&relation.status_before_invalidation).bind(now).bind(&relation.id)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

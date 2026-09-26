mod deleted;
mod history;
mod read;
mod write;

pub(crate) use deleted::{
    mark_restored, merge_restored_snapshot, prepare_restore_batch, prepare_restore_root,
    remove_tombstones, store_deletion_in_transaction, DeletionRecord,
};
pub(crate) use history::invalidate_for_nodes as invalidate_history_for_nodes;
pub(crate) use write::remove_transferred_structure_nodes_in_transaction;

use sqlx::{Sqlite, Transaction};

use super::{
    validate_content_coverage, validate_space_snapshot, RelationStyle, StructureLayout,
    StructureNode,
};
use crate::spaces::db;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct SpaceStructureSnapshot {
    pub structure: Vec<StructureNode>,
    pub layouts: Vec<StructureLayout>,
    pub relation_styles: Vec<RelationStyle>,
}

pub(crate) async fn initialize_empty_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let result = sqlx::query(
        "INSERT INTO spaces_edit_state(space_id,revision,updated_at) \
         SELECT id,0,$2 FROM spaces WHERE id=$1 \
         AND NOT EXISTS(SELECT 1 FROM spaces_nodes WHERE space_id=$1) \
         AND NOT EXISTS(SELECT 1 FROM spaces_structure_nodes WHERE space_id=$1) \
         ON CONFLICT(space_id) DO NOTHING",
    )
    .bind(space_id)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if result.rows_affected() != 1 {
        return Err("Space structure is already initialized or contains legacy data".into());
    }
    Ok(())
}

pub(crate) async fn load_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(SpaceStructureSnapshot, i64), String> {
    let revision = require_revision(tx, space_id).await?;
    let structure = read::structure(tx, space_id).await?;
    let layouts = read::layouts(tx, space_id).await?;
    let relation_styles = read::relation_styles(tx, space_id).await?;
    validate_space_snapshot(&structure, &layouts)?;
    let active_nodes = read::active_node_ids(tx, space_id).await?;
    validate_content_coverage(&structure, &active_nodes)?;
    read::validate_relation_styles(tx, space_id, &relation_styles).await?;
    Ok((
        SpaceStructureSnapshot {
            structure,
            layouts,
            relation_styles,
        },
        revision,
    ))
}

pub(crate) async fn save_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    base_revision: i64,
    snapshot: &SpaceStructureSnapshot,
) -> Result<i64, String> {
    validate_space_snapshot(&snapshot.structure, &snapshot.layouts)?;
    let active_nodes = read::active_node_ids(tx, space_id).await?;
    validate_content_coverage(&snapshot.structure, &active_nodes)?;
    read::validate_relation_styles(tx, space_id, &snapshot.relation_styles).await?;
    let current_revision = require_revision(tx, space_id).await?;
    if base_revision != current_revision {
        return Err("space-edit-revision-conflict: Space edit revision conflict".into());
    }
    write::replace_active(tx, space_id, snapshot).await?;
    let next_revision = base_revision
        .checked_add(1)
        .ok_or("Space edit revision overflow")?;
    let now = crate::clock::unix_time_millis()?;
    let result = sqlx::query(
        "UPDATE spaces_edit_state SET revision=$1,updated_at=$2 WHERE space_id=$3 AND revision=$4",
    )
    .bind(next_revision)
    .bind(now)
    .bind(space_id)
    .bind(base_revision)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if result.rows_affected() != 1 {
        return Err("space-edit-revision-conflict: Space edit revision conflict".into());
    }
    Ok(next_revision)
}

async fn require_revision(tx: &mut Transaction<'_, Sqlite>, space_id: &str) -> Result<i64, String> {
    sqlx::query_scalar("SELECT revision FROM spaces_edit_state WHERE space_id=$1")
        .bind(space_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?
        .ok_or_else(|| "Space structure is not initialized; legacy data must be reset".into())
}

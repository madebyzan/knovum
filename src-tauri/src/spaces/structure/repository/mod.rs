use crate::foundation::app_error::AppError;
mod history;
mod read;
pub(crate) mod rows;
mod write;

pub(crate) use history::invalidate_for_nodes as invalidate_history_for_nodes;

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
    let now = crate::foundation::clock::unix_time_millis()?;
    let edit_state_write = sqlx::query(
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
    if edit_state_write.rows_affected() != 1 {
        return Err("Space structure is already initialized or contains legacy data".into());
    }
    Ok(())
}

/// Reads the Space and proves it is coherent. Every writer loads through here, and a replay that
/// wrote only the rows one record named validates its result the same way.
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
) -> Result<i64, AppError> {
    validate_space_snapshot(&snapshot.structure, &snapshot.layouts)?;
    let active_nodes = read::active_node_ids(tx, space_id).await?;
    validate_content_coverage(&snapshot.structure, &active_nodes)?;
    read::validate_relation_styles(tx, space_id, &snapshot.relation_styles).await?;
    let current_revision = require_revision(tx, space_id).await?;
    if base_revision != current_revision {
        return Err(AppError::new(
            "space_edit_revision_conflict",
            "Space edit revision conflict",
        ));
    }
    write::persist_snapshot(tx, space_id, snapshot).await?;
    advance_revision(tx, space_id, base_revision).await
}

/// Publishes a write that already happened. A revision is the Space's edit identity, so it moves
/// exactly once per committed edit whether that edit was written as a snapshot or as rows.
pub(crate) async fn advance_revision(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    base_revision: i64,
) -> Result<i64, AppError> {
    let next_revision = base_revision
        .checked_add(1)
        .ok_or("Space edit revision overflow")?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let edit_state_write = sqlx::query(
        "UPDATE spaces_edit_state SET revision=$1,updated_at=$2 WHERE space_id=$3 AND revision=$4",
    )
    .bind(next_revision)
    .bind(now)
    .bind(space_id)
    .bind(base_revision)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if edit_state_write.rows_affected() != 1 {
        return Err(AppError::new(
            "space_edit_revision_conflict",
            "Space edit revision conflict",
        ));
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

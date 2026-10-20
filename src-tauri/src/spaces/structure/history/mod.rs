//! Undo and redo for a Space.
//!
//! An edit records the rows it changed, not the Space it produced, and a replay writes one side of
//! those rows back. Everything else in the Space is left exactly as it is, which is both the only
//! affordable way to keep a hundred edits per Space and the correct one: a card resized to fit its
//! media after the edit stays resized when an unrelated move is undone.

use crate::foundation::app_error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};

use super::repository::{self, SpaceStructureSnapshot};
use crate::spaces::db;

mod group;
mod patch;
mod replay;

pub(crate) use patch::HistoryState;
pub(in crate::spaces) use replay::apply;

/// How many edits a Space remembers. It bounds undo depth and, with it, every record whose only
/// purpose is to serve a replay of those edits.
pub(super) const RETAINED_EDITS: i64 = 100;

#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelationLifecycle {
    status: String,
    status_before_invalidation: Option<String>,
}

struct HistoryInsert<'a> {
    pub space_id: &'a str,
    pub patch: &'a patch::HistoryPatch,
    pub group_id: Option<&'a str>,
    pub now: i64,
}

pub(crate) async fn capture(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &SpaceStructureSnapshot,
) -> Result<HistoryState, String> {
    patch::capture_state(tx, space_id, snapshot).await
}

pub(crate) async fn record(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    before: &HistoryState,
    after: &HistoryState,
) -> Result<(), String> {
    clear_redo(tx, space_id).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    insert(
        tx,
        HistoryInsert {
            space_id,
            patch: &patch::diff(before, after),
            group_id: None,
            now,
        },
    )
    .await?;
    prune(tx, space_id).await
}

pub(crate) async fn record_group(
    tx: &mut Transaction<'_, Sqlite>,
    group_id: &str,
    entries: &[(&str, &HistoryState, &HistoryState)],
) -> Result<(), AppError> {
    group::record(tx, group_id, entries).await
}

async fn clear_redo(tx: &mut Transaction<'_, Sqlite>, space_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM spaces_edit_history WHERE space_id=$1 AND undone=1")
        .bind(space_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

async fn insert(tx: &mut Transaction<'_, Sqlite>, entry: HistoryInsert<'_>) -> Result<(), String> {
    let HistoryInsert {
        space_id,
        patch,
        group_id,
        now,
    } = entry;
    sqlx::query(
        "INSERT INTO spaces_edit_history(space_id,patch,history_group_id,undone,created_at) \
         VALUES($1,$2,$3,0,$4)",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(patch))
    .bind(group_id)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

async fn prune(tx: &mut Transaction<'_, Sqlite>, space_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM spaces_edit_history WHERE space_id=$1 AND id NOT IN (SELECT id FROM spaces_edit_history WHERE space_id=$1 ORDER BY id DESC LIMIT $2)").bind(space_id).bind(RETAINED_EDITS).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}

/// Restores the lifecycle of the relations this record changed. A relation row itself comes and
/// goes with the nodes it connects; only its status is history's to put back.
async fn apply_relations(
    tx: &mut Transaction<'_, Sqlite>,
    entry: &patch::HistoryPatch,
    side: patch::Side,
) -> Result<(), String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    for change in &entry.relations {
        let Some(lifecycle) = side.row(change) else {
            continue;
        };
        sqlx::query("UPDATE spaces_relations SET status=$1,status_before_invalidation=$2,updated_at=$3 WHERE id=$4")
            .bind(&lifecycle.status).bind(&lifecycle.status_before_invalidation).bind(now).bind(&change.id)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn apply_relation_styles(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    entry: &patch::HistoryPatch,
    side: patch::Side,
) -> Result<(), String> {
    let mut removed = Vec::new();
    let mut written = Vec::new();
    for change in &entry.relation_styles {
        match side.row(change) {
            Some(color) => written.push(super::RelationStyle {
                relation_id: change.id.clone(),
                color: color.clone(),
            }),
            None => removed.push(change.id.clone()),
        }
    }
    repository::rows::remove_relation_styles(tx, space_id, &removed).await?;
    repository::rows::write_relation_styles(tx, space_id, &written).await
}

/// Writes the structure rows one side of a record names. Every row the record touches leaves the
/// live set first: sibling order is unique among live rows, so a reorder cannot be applied one row
/// at a time without vacating the range it moves through.
async fn apply_structure(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    entry: &patch::HistoryPatch,
    side: patch::Side,
    current: &SpaceStructureSnapshot,
) -> Result<(), String> {
    let layouts = current
        .layouts
        .iter()
        .map(|layout| (layout.structure_node_id(), layout))
        .collect::<std::collections::HashMap<_, _>>();
    let mut rows = Vec::new();
    for change in &entry.structure {
        let Some(row) = side.row(change) else {
            continue;
        };
        let Some(layout) =
            patch::layout_to_write(change, side, layouts.get(change.id.as_str()).copied())
        else {
            continue;
        };
        rows.push(repository::rows::StructureRow {
            node: row.node.clone(),
            layout,
        });
    }
    repository::rows::write(tx, space_id, &rows).await
}

/// Content that comes back may show media whose intrinsic shape changed while it was away, so the
/// cards this record restored are refitted — and only those: the rest of the Space was not touched.
async fn fit_restored_content(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &mut SpaceStructureSnapshot,
    node_ids: &[String],
) -> Result<(), AppError> {
    if node_ids.is_empty()
        || !crate::spaces::nodes::fit_content_sizes_in_snapshot(tx, space_id, snapshot, node_ids)
            .await?
    {
        return Ok(());
    }
    let restored = snapshot
        .structure
        .iter()
        .filter_map(|node| match node {
            super::StructureNode::Content { id, node_id, .. } if node_ids.contains(node_id) => {
                Some(id.as_str())
            }
            _ => None,
        })
        .collect::<std::collections::HashSet<_>>();
    // The refit happens after the Space was proved coherent, so the sizes it produced are proved
    // before they are written rather than after.
    super::validate_space_snapshot(&snapshot.structure, &snapshot.layouts)?;
    let layouts = snapshot
        .layouts
        .iter()
        .filter(|layout| restored.contains(layout.structure_node_id()))
        .collect::<Vec<_>>();
    repository::rows::write_layouts(tx, space_id, &layouts).await?;
    Ok(())
}

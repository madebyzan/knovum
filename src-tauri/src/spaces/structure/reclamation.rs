//! Deleting content only soft-deletes it, so undo can bring it back. History keeps the last 100
//! edits per Space and drops its redo branch on a new edit; whatever no retained entry can restore,
//! and no live Portal or object still points at, is erased here together with the files only it
//! referenced. One pass runs when the Space runtime starts.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use sqlx::types::Json;
use sqlx::{Row, Sqlite, Transaction};
use tauri::AppHandle;

use crate::spaces::{assets, db, knowledge};

type IdsBySpace = BTreeMap<String, Vec<String>>;

/// Live Portals never target these: every structure save rejects a Portal to a deleted container.
const UNREACHABLE_STRUCTURE: &str = "WITH reachable(id) AS ( \
     SELECT json_extract(s.value,'$.id') FROM spaces_edit_history h, json_each(h.before_snapshot,'$.structure') s \
     UNION SELECT json_extract(s.value,'$.id') FROM spaces_edit_history h, json_each(h.after_snapshot,'$.structure') s) \
     SELECT space_id,id FROM spaces_structure_nodes \
     WHERE deleted_at IS NOT NULL AND id NOT IN (SELECT id FROM reachable) ORDER BY space_id,id";

const UNREACHABLE_NODES: &str = "WITH reachable(id) AS ( \
     SELECT n.value FROM spaces_edit_history h, json_each(h.before_node_ids) n \
     UNION SELECT n.value FROM spaces_edit_history h, json_each(h.after_node_ids) n) \
     SELECT space_id,id FROM spaces_nodes \
     WHERE deleted_at IS NOT NULL AND id NOT IN (SELECT id FROM reachable) ORDER BY space_id,id";

pub(crate) fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = reclaim(&app).await {
            log::error!("Soft-deleted Space content could not be reclaimed: {error}");
        }
    });
}

async fn reclaim(app: &AppHandle) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let asset_candidates = reclaim_in_transaction(&mut tx).await?;
    tx.commit().await.map_err(db::sql_error)?;
    knowledge::retrieval::invalidation_outbox::settle(app, pool).await;
    let removable = assets::unreferenced_paths(pool, asset_candidates).await?;
    assets::commands::remove_assets(app.clone(), removable)
        .await
        .map_err(|error| error.to_string())
}

/// Erases unreachable soft-deleted rows and returns the asset paths they referenced; the caller
/// removes those files after commit once no durable state references them.
pub(super) async fn reclaim_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<BTreeSet<String>, String> {
    let structure = ids_by_space(tx, UNREACHABLE_STRUCTURE).await?;
    let nodes = unreferenced_nodes(tx).await?;
    let mut asset_candidates = BTreeSet::new();
    for (space_id, structure_node_ids) in &structure {
        asset_candidates.extend(layout_backgrounds(tx, space_id, structure_node_ids).await?);
    }
    for (space_id, node_ids) in &nodes {
        let node_ids: HashSet<String> = node_ids.iter().cloned().collect();
        asset_candidates.extend(assets::referenced_paths_for_nodes(tx, space_id, &node_ids).await?);
    }
    // Structure rows reference their content, so they leave first.
    for (space_id, structure_node_ids) in &structure {
        erase_structure(tx, space_id, structure_node_ids).await?;
    }
    for (space_id, node_ids) in &nodes {
        erase_nodes(tx, space_id, node_ids).await?;
    }
    Ok(asset_candidates)
}

async fn ids_by_space(tx: &mut Transaction<'_, Sqlite>, sql: &str) -> Result<IdsBySpace, String> {
    let rows: Vec<(String, String)> = sqlx::query_as(sql)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    let mut ids = IdsBySpace::new();
    for (space_id, id) in rows {
        ids.entry(space_id).or_default().push(id);
    }
    Ok(ids)
}

/// Content still targeted by a live Portal or object field stays: erasing it would leave that
/// reference dangling. It becomes reclaimable once the reference is gone.
async fn unreferenced_nodes(tx: &mut Transaction<'_, Sqlite>) -> Result<IdsBySpace, String> {
    let mut nodes = ids_by_space(tx, UNREACHABLE_NODES).await?;
    if nodes.is_empty() {
        return Ok(nodes);
    }
    let referenced = live_references(tx).await?;
    for node_ids in nodes.values_mut() {
        node_ids.retain(|id| !referenced.contains(id));
    }
    nodes.retain(|_, node_ids| !node_ids.is_empty());
    Ok(nodes)
}

async fn live_references(tx: &mut Transaction<'_, Sqlite>) -> Result<HashSet<String>, String> {
    let mut referenced: HashSet<String> = sqlx::query_scalar(
        "SELECT json_extract(data,'$.targetNodeId') FROM spaces_nodes \
         WHERE kind='portal' AND deleted_at IS NULL AND json_type(data,'$.targetNodeId')='text'",
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .into_iter()
    .collect();
    let objects = sqlx::query(
        "SELECT n.data,t.fields FROM spaces_nodes n \
         LEFT JOIN spaces_object_types t ON t.id=n.object_type_id \
         WHERE n.kind='object' AND n.deleted_at IS NULL",
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for row in objects {
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let fields: Option<String> = row.try_get("fields").map_err(db::sql_error)?;
        let data = serde_json::from_str(&raw).map_err(db::json_error)?;
        referenced.extend(knowledge::object_references::collect_node_ids(
            fields.as_deref(),
            &data,
        )?);
    }
    Ok(referenced)
}

async fn layout_backgrounds(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure_node_ids: &[String],
) -> Result<Vec<String>, String> {
    sqlx::query_scalar(
        "SELECT json_extract(data,'$.appearance.background') FROM spaces_structure_layouts \
         WHERE space_id=$1 AND structure_node_id IN (SELECT value FROM json_each($2)) \
         AND json_type(data,'$.appearance.background')='text'",
    )
    .bind(space_id)
    .bind(Json(structure_node_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)
}

async fn erase_structure(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure_node_ids: &[String],
) -> Result<(), String> {
    sqlx::query(
        "DELETE FROM spaces_structure_layouts \
         WHERE space_id=$1 AND structure_node_id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(Json(structure_node_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "DELETE FROM spaces_structure_nodes \
         WHERE space_id=$1 AND deleted_at IS NOT NULL AND id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(Json(structure_node_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

async fn erase_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_ids: &[String],
) -> Result<(), String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    knowledge::retrieval::invalidation_outbox::enqueue_nodes(tx, space_id, node_ids, now).await?;
    knowledge::grounding::references::detach_nodes(tx, space_id, node_ids).await?;
    let relations: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_relations WHERE space_id=$1 \
         AND (source_id IN (SELECT value FROM json_each($2)) \
           OR target_id IN (SELECT value FROM json_each($2)))",
    )
    .bind(space_id)
    .bind(Json(node_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let provenance: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_provenance WHERE space_id=$1 \
         AND (source_node_id IN (SELECT value FROM json_each($2)) \
           OR (target_kind='node' AND target_id IN (SELECT value FROM json_each($2))) \
           OR (target_kind='relation' AND target_id IN (SELECT value FROM json_each($3))))",
    )
    .bind(space_id)
    .bind(Json(node_ids))
    .bind(Json(&relations))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "DELETE FROM spaces_knowledge_events \
         WHERE (entity_kind='node' AND entity_id IN (SELECT value FROM json_each($1))) \
            OR (entity_kind='relation' AND entity_id IN (SELECT value FROM json_each($2))) \
            OR (entity_kind='provenance' AND entity_id IN (SELECT value FROM json_each($3)))",
    )
    .bind(Json(node_ids))
    .bind(Json(&relations))
    .bind(Json(&provenance))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("DELETE FROM spaces_provenance WHERE id IN (SELECT value FROM json_each($1))")
        .bind(Json(&provenance))
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    sqlx::query("DELETE FROM spaces_relations WHERE id IN (SELECT value FROM json_each($1))")
        .bind(Json(&relations))
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    sqlx::query(
        "DELETE FROM spaces_nodes \
         WHERE space_id=$1 AND deleted_at IS NOT NULL AND id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(Json(node_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

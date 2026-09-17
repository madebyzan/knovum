use std::collections::{BTreeSet, HashSet};

use sqlx::Row;
use tauri::{AppHandle, Runtime};

use super::contracts::DeleteDeletedNodesRequest;
use super::validation;
use crate::spaces::{db, knowledge};

pub async fn delete_nodes<R: Runtime>(
    app: &AppHandle<R>,
    request: DeleteDeletedNodesRequest,
) -> Result<Vec<String>, String> {
    if request.selections.is_empty() {
        return Ok(Vec::new());
    }
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let mut spaces = HashSet::new();
    let mut selections = Vec::new();
    let mut erased = HashSet::new();
    for selection in &request.selections {
        if selection.space_id.trim().is_empty() || !spaces.insert(&selection.space_id) {
            return Err("Permanent deletion requires unique, non-empty Space identifiers".into());
        }
        let nodes = validation::deletable_nodes(&mut tx, selection).await?;
        let ids: HashSet<String> = nodes.into_iter().map(|(id, _)| id).collect();
        erased.extend(ids.iter().cloned());
        selections.push((&selection.space_id, ids));
    }
    for (space_id, _) in &selections {
        validation::reject_persistent_dependents(&mut tx, space_id, &erased).await?;
    }
    let now = crate::clock::unix_time_millis()?;
    let mut paths = BTreeSet::new();
    for (space_id, ids) in selections {
        paths.extend(erase_selection(&mut tx, space_id, &ids, now).await?);
    }
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::knowledge::retrieval::invalidation_outbox::drain_and_report(app, pool).await;
    crate::spaces::assets::unreferenced_paths(pool, paths).await
}

async fn erase_selection(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_ids: &HashSet<String>,
    now: i64,
) -> Result<BTreeSet<String>, String> {
    let paths = crate::spaces::assets::referenced_paths_for_nodes(tx, space_id, node_ids).await?;

    let relation_ids = related_relation_ids(tx, space_id, node_ids).await?;
    let provenance_ids = related_provenance_ids(tx, space_id, node_ids, &relation_ids).await?;

    let detached_node_ids = node_ids.iter().cloned().collect::<Vec<_>>();
    crate::spaces::knowledge::retrieval::invalidation_outbox::enqueue_nodes(
        tx,
        space_id,
        &detached_node_ids,
        now,
    )
    .await?;
    knowledge::grounding::references::detach_nodes(tx, space_id, &detached_node_ids).await?;
    delete_events(tx, "node", node_ids).await?;
    delete_events(tx, "relation", &relation_ids).await?;
    delete_events(tx, "provenance", &provenance_ids).await?;
    delete_by_ids(tx, "spaces_provenance", &provenance_ids).await?;
    delete_by_ids(tx, "spaces_relations", &relation_ids).await?;
    for id in node_ids {
        sqlx::query(
            "DELETE FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NOT NULL",
        )
        .bind(id)
        .bind(space_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(paths)
}

async fn related_relation_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    nodes: &HashSet<String>,
) -> Result<HashSet<String>, String> {
    let rows = sqlx::query("SELECT id,source_id,target_id FROM spaces_relations WHERE space_id=$1")
        .bind(space_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    let mut ids = HashSet::new();
    for row in rows {
        let source: String = row.try_get("source_id").map_err(db::sql_error)?;
        let target: String = row.try_get("target_id").map_err(db::sql_error)?;
        if nodes.contains(&source) || nodes.contains(&target) {
            ids.insert(row.try_get("id").map_err(db::sql_error)?);
        }
    }
    Ok(ids)
}

async fn related_provenance_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    nodes: &HashSet<String>,
    relations: &HashSet<String>,
) -> Result<HashSet<String>, String> {
    let rows = sqlx::query(
        "SELECT id,source_node_id,target_kind,target_id FROM spaces_provenance WHERE space_id=$1",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut ids = HashSet::new();
    for row in rows {
        let source: String = row.try_get("source_node_id").map_err(db::sql_error)?;
        let kind: String = row.try_get("target_kind").map_err(db::sql_error)?;
        let target: String = row.try_get("target_id").map_err(db::sql_error)?;
        if nodes.contains(&source)
            || (kind == "node" && nodes.contains(&target))
            || (kind == "relation" && relations.contains(&target))
        {
            ids.insert(row.try_get("id").map_err(db::sql_error)?);
        }
    }
    Ok(ids)
}

async fn delete_events(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    kind: &str,
    ids: &HashSet<String>,
) -> Result<(), String> {
    for id in ids {
        sqlx::query("DELETE FROM spaces_knowledge_events WHERE entity_kind=$1 AND entity_id=$2")
            .bind(kind)
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

async fn delete_by_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    table: &str,
    ids: &HashSet<String>,
) -> Result<(), String> {
    let sql = match table {
        "spaces_provenance" => "DELETE FROM spaces_provenance WHERE id=$1",
        "spaces_relations" => "DELETE FROM spaces_relations WHERE id=$1",
        _ => return Err("Unsupported permanent-delete table".into()),
    };
    for id in ids {
        sqlx::query(sql)
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

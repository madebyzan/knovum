use std::collections::HashSet;

use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::{db, structure::StructureNode};

mod dependents;
mod history;

pub(super) use dependents::reject_object_dependents;

pub(super) async fn erase(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    batch_id: &str,
    structure: &[StructureNode],
    node_ids: &HashSet<String>,
    remaining_roots: Vec<String>,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let nodes = node_ids.iter().cloned().collect::<Vec<_>>();
    crate::spaces::knowledge::retrieval::invalidation_outbox::enqueue_nodes(
        tx, space_id, &nodes, now,
    )
    .await?;
    crate::spaces::knowledge::grounding::references::detach_nodes(tx, space_id, &nodes).await?;
    let relations = relation_ids(tx, space_id, node_ids).await?;
    let provenance = provenance_ids(tx, space_id, node_ids, &relations).await?;
    history::invalidate(tx, space_id, structure, node_ids).await?;
    delete_events(tx, "node", node_ids).await?;
    delete_events(tx, "relation", &relations).await?;
    delete_events(tx, "provenance", &provenance).await?;
    delete_ids(tx, "spaces_provenance", &provenance).await?;
    delete_ids(tx, "spaces_relations", &relations).await?;
    super::super::super::repository::remove_tombstones(tx, space_id, structure).await?;
    update_batch(tx, batch_id, structure, &relations, remaining_roots).await?;
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
    Ok(())
}

async fn relation_ids(
    tx: &mut Transaction<'_, Sqlite>,
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

async fn provenance_ids(
    tx: &mut Transaction<'_, Sqlite>,
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

async fn update_batch(
    tx: &mut Transaction<'_, Sqlite>,
    batch_id: &str,
    structure: &[StructureNode],
    relations: &HashSet<String>,
    remaining_roots: Vec<String>,
) -> Result<(), String> {
    if remaining_roots.is_empty() {
        sqlx::query("DELETE FROM spaces_deleted_structure_batches WHERE id=$1")
            .bind(batch_id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
        return Ok(());
    }
    for structure_node in structure {
        sqlx::query("DELETE FROM spaces_deleted_structure_layouts WHERE batch_id=$1 AND structure_node_id=$2")
            .bind(batch_id)
            .bind(structure_node.id())
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
        sqlx::query(
            "DELETE FROM spaces_deleted_structure_nodes WHERE batch_id=$1 AND structure_node_id=$2",
        )
        .bind(batch_id)
        .bind(structure_node.id())
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    for relation in relations {
        sqlx::query(
            "DELETE FROM spaces_deleted_structure_relations WHERE batch_id=$1 AND relation_id=$2",
        )
        .bind(batch_id)
        .bind(relation)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    sqlx::query(
        "UPDATE spaces_deleted_structure_batches SET root_structure_node_ids=$1 WHERE id=$2",
    )
    .bind(sqlx::types::Json(&remaining_roots))
    .bind(batch_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

async fn delete_events(
    tx: &mut Transaction<'_, Sqlite>,
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

async fn delete_ids(
    tx: &mut Transaction<'_, Sqlite>,
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

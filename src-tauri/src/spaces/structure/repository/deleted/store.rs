use sqlx::{Sqlite, Transaction};

use super::super::super::{StructureLayout, StructureNode};
use crate::spaces::db;

pub(crate) struct DeletionBatch<'a> {
    pub space_id: &'a str,
    pub batch_id: &'a str,
    pub roots: &'a [String],
    pub structure: &'a [StructureNode],
    pub layouts: &'a [StructureLayout],
    pub node_ids: &'a [String],
    pub deleted_at: i64,
}

pub(crate) async fn store_deletion_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    record: DeletionBatch<'_>,
) -> Result<(), String> {
    let roots = serde_json::to_string(record.roots).map_err(db::json_error)?;
    sqlx::query("INSERT INTO spaces_deleted_structure_batches(id,space_id,root_structure_node_ids,deleted_at) VALUES($1,$2,$3,$4)")
        .bind(record.batch_id).bind(record.space_id).bind(roots).bind(record.deleted_at)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    store_structure(tx, record.batch_id, record.structure).await?;
    store_layouts(tx, record.batch_id, record.layouts).await?;
    let node_ids = serde_json::to_string(record.node_ids).map_err(db::json_error)?;
    sqlx::query("INSERT INTO spaces_deleted_structure_relations(batch_id,relation_id,status,status_before_invalidation,color) SELECT $1,r.id,r.status,r.status_before_invalidation,s.color FROM spaces_relations r LEFT JOIN spaces_relation_styles s ON s.space_id=r.space_id AND s.relation_id=r.id WHERE r.space_id=$2 AND (r.source_id IN (SELECT value FROM json_each($3)) OR r.target_id IN (SELECT value FROM json_each($3)))")
        .bind(record.batch_id).bind(record.space_id).bind(node_ids).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}

async fn store_structure(
    tx: &mut Transaction<'_, Sqlite>,
    batch_id: &str,
    structure: &[StructureNode],
) -> Result<(), String> {
    for structure_node in structure {
        let data = serde_json::to_string(structure_node).map_err(db::json_error)?;
        sqlx::query(
            "INSERT INTO spaces_deleted_structure_nodes(batch_id,structure_node_id,data) VALUES($1,$2,$3)",
        )
        .bind(batch_id)
        .bind(structure_node.id())
        .bind(data)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}

async fn store_layouts(
    tx: &mut Transaction<'_, Sqlite>,
    batch_id: &str,
    layouts: &[StructureLayout],
) -> Result<(), String> {
    for layout in layouts {
        let data = serde_json::to_string(layout).map_err(db::json_error)?;
        sqlx::query(
            "INSERT INTO spaces_deleted_structure_layouts(batch_id,structure_node_id,data) VALUES($1,$2,$3)",
        )
        .bind(batch_id)
        .bind(layout.structure_node_id())
        .bind(data)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}

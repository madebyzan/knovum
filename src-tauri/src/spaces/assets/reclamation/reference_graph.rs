use std::collections::{BTreeSet, HashSet};

use serde_json::Value;
use sqlx::Row;

use super::super::references::collect_node_asset_paths;
use crate::spaces::db;

pub(super) async fn all(pool: &sqlx::SqlitePool) -> Result<BTreeSet<String>, String> {
    let mut paths = BTreeSet::new();
    let rows = sqlx::query("SELECT data FROM spaces_nodes")
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?;
    collect_node_rows(rows, &mut paths)?;
    let layout_paths = sqlx::query_scalar::<_, String>(
        "SELECT json_extract(data,'$.appearance.background') FROM spaces_structure_layouts \
         WHERE json_type(data,'$.appearance.background')='text' \
         UNION ALL \
         SELECT json_extract(l.data,'$.appearance.background') \
         FROM spaces_deleted_structure_layouts l \
         JOIN spaces_deleted_structure_batches b ON b.id=l.batch_id \
         WHERE b.restored_at IS NULL AND json_type(l.data,'$.appearance.background')='text'",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    paths.extend(layout_paths);
    let thumbnails = sqlx::query_scalar::<_, String>(
        "SELECT thumbnail_path FROM spaces WHERE thumbnail_path IS NOT NULL \
         UNION ALL SELECT thumbnail_path FROM spaces_scenes WHERE thumbnail_path IS NOT NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    paths.extend(thumbnails);
    let checkpoints = sqlx::query_scalar::<_, String>(
        "SELECT json_extract(c.resource,'$.path') FROM spaces_document_visual_checkpoints c \
         JOIN spaces_enrichment_jobs j ON j.id=c.job_id JOIN spaces_nodes n ON n.id=j.node_id AND n.space_id=j.space_id \
         WHERE c.resource IS NOT NULL AND j.status IN ('pending','running','failed','awaiting_configuration') \
           AND json_extract(n.data,'$.sourceRevision')=j.source_revision",
    ).fetch_all(pool).await.map_err(db::sql_error)?;
    paths.extend(checkpoints);
    Ok(paths)
}

pub(crate) async fn for_nodes(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_ids: &HashSet<String>,
) -> Result<BTreeSet<String>, String> {
    if node_ids.is_empty() {
        return Ok(BTreeSet::new());
    }
    let ids = serde_json::to_string(node_ids).map_err(db::json_error)?;
    let rows = sqlx::query(
        "SELECT data FROM spaces_nodes WHERE space_id=$1 AND id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(&ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut paths = BTreeSet::new();
    collect_node_rows(rows, &mut paths)?;
    let checkpoints = sqlx::query_scalar::<_, String>(
        "SELECT json_extract(c.resource,'$.path') FROM spaces_document_visual_checkpoints c \
         JOIN spaces_enrichment_jobs j ON j.id=c.job_id WHERE c.resource IS NOT NULL \
           AND j.space_id=$1 AND j.node_id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(&ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    paths.extend(checkpoints);
    let layout_paths = sqlx::query_scalar::<_, String>(
        "SELECT json_extract(l.data,'$.appearance.background') \
         FROM spaces_structure_layouts l \
         JOIN spaces_structure_nodes i ON i.space_id=l.space_id AND i.id=l.structure_node_id \
         WHERE l.space_id=$1 AND i.node_id IN (SELECT value FROM json_each($2)) \
           AND json_type(l.data,'$.appearance.background')='text' \
         UNION ALL \
         SELECT json_extract(l.data,'$.appearance.background') \
         FROM spaces_deleted_structure_layouts l \
         JOIN spaces_deleted_structure_nodes i ON i.batch_id=l.batch_id AND i.structure_node_id=l.structure_node_id \
         JOIN spaces_deleted_structure_batches b ON b.id=l.batch_id \
         WHERE b.space_id=$1 AND b.restored_at IS NULL \
           AND json_extract(i.data,'$.nodeId') IN (SELECT value FROM json_each($2)) \
           AND json_type(l.data,'$.appearance.background')='text'",
    )
    .bind(space_id)
    .bind(ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    paths.extend(layout_paths);
    Ok(paths)
}

pub(crate) async fn for_space(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
) -> Result<BTreeSet<String>, String> {
    let rows = sqlx::query("SELECT data FROM spaces_nodes WHERE space_id=$1")
        .bind(space_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    let mut paths = BTreeSet::new();
    collect_node_rows(rows, &mut paths)?;
    let checkpoints = sqlx::query_scalar::<_, String>(
        "SELECT json_extract(c.resource,'$.path') FROM spaces_document_visual_checkpoints c \
         JOIN spaces_enrichment_jobs j ON j.id=c.job_id WHERE j.space_id=$1 AND c.resource IS NOT NULL",
    ).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    paths.extend(checkpoints);
    let layout_paths = sqlx::query_scalar::<_, String>(
        "SELECT json_extract(data,'$.appearance.background') FROM spaces_structure_layouts \
         WHERE space_id=$1 AND json_type(data,'$.appearance.background')='text' \
         UNION ALL \
         SELECT json_extract(l.data,'$.appearance.background') \
         FROM spaces_deleted_structure_layouts l \
         JOIN spaces_deleted_structure_batches b ON b.id=l.batch_id \
         WHERE b.space_id=$1 AND b.restored_at IS NULL \
           AND json_type(l.data,'$.appearance.background')='text'",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    paths.extend(layout_paths);
    let thumbnails = sqlx::query_scalar::<_, String>(
        "SELECT thumbnail_path FROM spaces WHERE id=$1 AND thumbnail_path IS NOT NULL \
         UNION ALL SELECT thumbnail_path FROM spaces_scenes WHERE space_id=$1 AND thumbnail_path IS NOT NULL",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    paths.extend(thumbnails);
    Ok(paths)
}

fn collect_node_rows(
    rows: Vec<sqlx::sqlite::SqliteRow>,
    paths: &mut BTreeSet<String>,
) -> Result<(), String> {
    for row in rows {
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let value: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        collect_node_asset_paths(&value, paths);
    }
    Ok(())
}

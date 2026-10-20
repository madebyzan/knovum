use std::collections::HashMap;

use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

/// A Portal records which Space its target lives in, so the Portals that point at moved content
/// are rewritten — and they are found by their target, not by scanning every Portal in the library.
pub(super) async fn update_targets(
    tx: &mut Transaction<'_, Sqlite>,
    owner_by_node_id: &HashMap<String, &str>,
    owner_by_container_id: &HashMap<&str, &str>,
    moved_nodes: &[String],
) -> Result<(), String> {
    let containers = owner_by_container_id.keys().copied().collect::<Vec<_>>();
    if moved_nodes.is_empty() && containers.is_empty() {
        return Ok(());
    }
    let rows = sqlx::query(
        "SELECT id,data FROM spaces_nodes WHERE kind='portal' AND deleted_at IS NULL AND ( \
           json_extract(data,'$.targetNodeId') IN (SELECT value FROM json_each($1)) \
           OR json_extract(data,'$.targetStructureNodeId') IN (SELECT value FROM json_each($2)))",
    )
    .bind(sqlx::types::Json(moved_nodes))
    .bind(sqlx::types::Json(&containers))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let mut data: serde_json::Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        let owner = data
            .get("targetNodeId")
            .and_then(serde_json::Value::as_str)
            .and_then(|target| owner_by_node_id.get(target))
            .or_else(|| {
                data.get("targetStructureNodeId")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|target| owner_by_container_id.get(target))
            });
        let Some(owner) = owner else { continue };
        data["targetSpaceId"] = serde_json::Value::String((*owner).to_owned());
        sqlx::query("UPDATE spaces_nodes SET data=$1 WHERE id=$2")
            .bind(sqlx::types::Json(&data))
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

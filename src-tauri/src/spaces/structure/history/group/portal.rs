use std::collections::HashMap;

use sqlx::{Row, Sqlite, Transaction};

use super::TargetState;
use crate::spaces::db;

pub(super) fn node_owners(targets: &[TargetState]) -> HashMap<&str, &str> {
    targets
        .iter()
        .flat_map(|target| {
            target
                .node_ids
                .iter()
                .map(move |id| (id.as_str(), target.space_id.as_str()))
        })
        .collect()
}

pub(super) fn structure_owners(targets: &[TargetState]) -> HashMap<&str, &str> {
    targets
        .iter()
        .flat_map(|target| {
            target
                .snapshot
                .structure
                .iter()
                .filter_map(move |structure_node| {
                    (structure_node.kind_name() != "content")
                        .then_some((structure_node.id(), target.space_id.as_str()))
                })
        })
        .collect()
}

pub(super) async fn update_targets(
    tx: &mut Transaction<'_, Sqlite>,
    node_owners: &HashMap<&str, &str>,
    structure_owners: &HashMap<&str, &str>,
) -> Result<(), String> {
    let rows =
        sqlx::query("SELECT id,data FROM spaces_nodes WHERE kind='portal' AND deleted_at IS NULL")
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
            .and_then(|target| node_owners.get(target))
            .or_else(|| {
                data.get("targetStructureNodeId")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|target| structure_owners.get(target))
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

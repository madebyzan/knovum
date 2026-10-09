use std::collections::HashMap;

use sqlx::{Row, Sqlite, Transaction};

use super::edit_contract::SpaceEditCommand;
use super::repository::SpaceStructureSnapshot;
use super::StructureNode;
use crate::spaces::db;

pub(super) async fn validate(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &SpaceStructureSnapshot,
    command: &SpaceEditCommand,
) -> Result<(), String> {
    let SpaceEditCommand::ContentUpdateHeights { updates } = command else {
        return Ok(());
    };
    if updates.is_empty() || updates.len() > 100_000 {
        return Err("Content height update count is invalid".into());
    }
    if updates.iter().any(|update| {
        update
            .content_revision
            .as_ref()
            .is_some_and(|revision| revision.is_empty() || revision.encode_utf16().count() > 128)
    }) {
        return Err("Content revision must contain 1 to 128 characters".into());
    }
    let nodes = snapshot
        .structure
        .iter()
        .filter_map(|structure_node| match structure_node {
            StructureNode::Content { id, node_id, .. } => Some((id.as_str(), node_id.as_str())),
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    let node_ids = updates
        .iter()
        .map(|update| {
            nodes
                .get(update.structure_node_id.as_str())
                .copied()
                .ok_or_else(|| {
                    format!(
                        "Content structure_node not found: {}",
                        update.structure_node_id
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let rows = sqlx::query(
        "SELECT id,kind,json_extract(data,'$.knowledgeRevision') AS content_revision \
         FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL \
         AND id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(&node_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let current = rows
        .into_iter()
        .map(|row| {
            Ok((
                row.try_get::<String, _>("id").map_err(db::sql_error)?,
                (
                    row.try_get::<String, _>("kind").map_err(db::sql_error)?,
                    row.try_get::<Option<String>, _>("content_revision")
                        .map_err(db::sql_error)?,
                ),
            ))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;
    for (update, node_id) in updates.iter().zip(node_ids) {
        let Some((kind, revision)) = current.get(node_id) else {
            return Err(format!("Content node no longer exists: {node_id}"));
        };
        if kind != "text" {
            return Err(format!(
                "Content height fitting is unsupported for: {node_id}"
            ));
        }
        if revision != &update.content_revision {
            return Err(format!(
                "Content height measurement is stale: {}",
                update.structure_node_id
            ));
        }
    }
    Ok(())
}

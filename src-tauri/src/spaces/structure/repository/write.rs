use std::collections::HashMap;

use sqlx::{Row, Sqlite, Transaction};

use super::super::contracts::StructureNode;
use super::{rows, SpaceStructureSnapshot};
use crate::spaces::db;

pub(super) async fn persist_snapshot(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &SpaceStructureSnapshot,
) -> Result<(), String> {
    let structure_ids = snapshot
        .structure
        .iter()
        .map(StructureNode::id)
        .collect::<Vec<_>>();
    let existing = existing_structure(tx, space_id, &structure_ids).await?;
    for structure_node in &snapshot.structure {
        if let Some((current, _deleted)) = existing.get(structure_node.id()) {
            if current.kind_name() != structure_node.kind_name() {
                return Err(format!(
                    "Structure node ID cannot be reused or change kind: {}",
                    structure_node.id()
                ));
            }
            if let StructureNode::Content {
                node_id: requested, ..
            } = structure_node
            {
                if !matches!(current, StructureNode::Content { node_id, .. } if node_id == requested)
                {
                    return Err(format!(
                        "Content reference cannot change for structure_node: {}",
                        structure_node.id()
                    ));
                }
            }
        }
    }
    let now = crate::foundation::clock::unix_time_millis()?;
    let moving_ids = snapshot
        .structure
        .iter()
        .filter(|node| {
            existing.get(node.id()).is_some_and(|(current, deleted)| {
                !deleted
                    && (current.parent_id() != node.parent_id() || current.order() != node.order())
            })
        })
        .map(StructureNode::id)
        .collect::<Vec<_>>();
    sqlx::query(
        "UPDATE spaces_structure_nodes SET deleted_at=$1 WHERE space_id=$2 AND deleted_at IS NULL \
         AND (id NOT IN (SELECT value FROM json_each($3)) OR id IN (SELECT value FROM json_each($4)))",
    )
    .bind(now)
    .bind(space_id)
    .bind(sqlx::types::Json(&structure_ids))
    .bind(sqlx::types::Json(&moving_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for structure_node in &snapshot.structure {
        if !existing
            .get(structure_node.id())
            .is_some_and(|(current, deleted)| !deleted && current == structure_node)
        {
            rows::upsert_structure_node(tx, space_id, structure_node).await?;
        }
    }
    rows::write_layouts(tx, space_id, &snapshot.layouts.iter().collect::<Vec<_>>()).await?;
    sqlx::query("DELETE FROM spaces_relation_styles WHERE space_id=$1 AND relation_id NOT IN (SELECT value FROM json_each($2))")
        .bind(space_id)
        .bind(sqlx::types::Json(snapshot.relation_styles.iter().map(|style| &style.relation_id).collect::<Vec<_>>()))
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    rows::write_relation_styles(tx, space_id, &snapshot.relation_styles).await
}

async fn existing_structure(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure_ids: &[&str],
) -> Result<HashMap<String, (StructureNode, bool)>, String> {
    let rows = sqlx::query(
        "SELECT id,kind,node_id,parent_id,order_index,title,locked,deleted_at FROM spaces_structure_nodes \
         WHERE space_id=$1 AND id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(structure_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            let deleted: Option<i64> = row.try_get("deleted_at").map_err(db::sql_error)?;
            let node = super::read::decode_structure_node(row)?;
            Ok((node.id().to_owned(), (node, deleted.is_some())))
        })
        .collect()
}

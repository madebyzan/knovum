use std::collections::HashMap;

use sqlx::{Row, Sqlite, Transaction};

use super::super::contract::StructureNode;
use super::SpaceStructureSnapshot;
use crate::spaces::db;

pub(super) async fn replace_active(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &SpaceStructureSnapshot,
) -> Result<(), String> {
    let existing = existing_structure(tx, space_id).await?;
    for structure_node in &snapshot.structure {
        if let Some((kind, _deleted, node_id)) = existing.get(structure_node.id()) {
            if kind != structure_node.kind_name() {
                return Err(format!(
                    "Structure node ID cannot be reused or change kind: {}",
                    structure_node.id()
                ));
            }
            if let StructureNode::Content {
                node_id: requested, ..
            } = structure_node
            {
                if node_id.as_deref() != Some(requested) {
                    return Err(format!(
                        "Content reference cannot change for structure_node: {}",
                        structure_node.id()
                    ));
                }
            }
        }
    }
    let now = crate::clock::unix_time_millis()?;
    sqlx::query(
        "UPDATE spaces_structure_nodes SET deleted_at=$1 WHERE space_id=$2 AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(space_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for structure_node in &snapshot.structure {
        upsert_structure_node(tx, space_id, structure_node).await?;
    }
    for layout in &snapshot.layouts {
        let data = serde_json::to_string(layout).map_err(db::json_error)?;
        sqlx::query(
            "INSERT INTO spaces_structure_layouts(space_id,structure_node_id,kind,data) VALUES($1,$2,$3,$4) \
             ON CONFLICT(space_id,structure_node_id) DO UPDATE SET data=excluded.data,kind=excluded.kind",
        )
        .bind(space_id)
        .bind(layout.structure_node_id())
        .bind(layout.kind_name())
        .bind(data)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    sqlx::query("DELETE FROM spaces_relation_styles WHERE space_id=$1")
        .bind(space_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    for style in &snapshot.relation_styles {
        sqlx::query(
            "INSERT INTO spaces_relation_styles(space_id,relation_id,color) VALUES($1,$2,$3)",
        )
        .bind(space_id)
        .bind(&style.relation_id)
        .bind(style.color.as_str())
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}

pub(crate) async fn remove_transferred_structure_nodes_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure_node_ids_deepest_first: &[String],
) -> Result<(), String> {
    for structure_node_id in structure_node_ids_deepest_first {
        sqlx::query(
            "DELETE FROM spaces_structure_layouts WHERE space_id=$1 AND structure_node_id=$2",
        )
        .bind(space_id)
        .bind(structure_node_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        let result = sqlx::query(
            "DELETE FROM spaces_structure_nodes WHERE space_id=$1 AND id=$2 AND deleted_at IS NULL",
        )
        .bind(space_id)
        .bind(structure_node_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if result.rows_affected() != 1 {
            return Err(format!(
                "Transferred structure node changed during transaction: {structure_node_id}"
            ));
        }
    }
    Ok(())
}

async fn existing_structure(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<HashMap<String, (String, bool, Option<String>)>, String> {
    let rows = sqlx::query(
        "SELECT id,kind,node_id,deleted_at FROM spaces_structure_nodes WHERE space_id=$1",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            let id: String = row.try_get("id").map_err(db::sql_error)?;
            let kind: String = row.try_get("kind").map_err(db::sql_error)?;
            let node_id: Option<String> = row.try_get("node_id").map_err(db::sql_error)?;
            let deleted: Option<i64> = row.try_get("deleted_at").map_err(db::sql_error)?;
            Ok((id, (kind, deleted.is_some(), node_id)))
        })
        .collect()
}

async fn upsert_structure_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure_node: &StructureNode,
) -> Result<(), String> {
    let (node_id, title) = match structure_node {
        StructureNode::Content { node_id, .. } => (Some(node_id.as_str()), None),
        StructureNode::Group { title, .. } | StructureNode::Section { title, .. } => {
            (None, Some(title.as_str()))
        }
    };
    let result = sqlx::query(
        "INSERT INTO spaces_structure_nodes \
         (id,space_id,kind,node_id,parent_id,order_index,title,locked,deleted_at) \
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,NULL) \
         ON CONFLICT(id) DO UPDATE SET parent_id=excluded.parent_id,order_index=excluded.order_index, \
         title=excluded.title,locked=excluded.locked,deleted_at=NULL \
         WHERE spaces_structure_nodes.space_id=excluded.space_id \
         AND spaces_structure_nodes.kind=excluded.kind",
    )
    .bind(structure_node.id())
    .bind(space_id)
    .bind(structure_node.kind_name())
    .bind(node_id)
    .bind(structure_node.parent_id())
    .bind(i64::from(structure_node.order()))
    .bind(title)
    .bind(structure_node.locked())
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if result.rows_affected() != 1 {
        return Err(format!(
            "Structure node ID belongs to another Space: {}",
            structure_node.id()
        ));
    }
    Ok(())
}

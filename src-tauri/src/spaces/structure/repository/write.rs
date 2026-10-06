use std::collections::HashMap;

use sqlx::{Row, Sqlite, Transaction};

use super::super::contract::StructureNode;
use super::SpaceStructureSnapshot;
use crate::spaces::db;

pub(super) async fn persist_snapshot(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &SpaceStructureSnapshot,
) -> Result<(), String> {
    let existing = existing_structure(tx, space_id).await?;
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
    let now = crate::clock::unix_time_millis()?;
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
    .bind(serde_json::to_string(&snapshot.structure.iter().map(StructureNode::id).collect::<Vec<_>>()).map_err(db::json_error)?)
    .bind(serde_json::to_string(&moving_ids).map_err(db::json_error)?)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for structure_node in &snapshot.structure {
        if !existing
            .get(structure_node.id())
            .is_some_and(|(current, deleted)| !deleted && current == structure_node)
        {
            upsert_structure_node(tx, space_id, structure_node).await?;
        }
    }
    for layout in &snapshot.layouts {
        let data = serde_json::to_string(layout).map_err(db::json_error)?;
        sqlx::query(
            "INSERT INTO spaces_structure_layouts(space_id,structure_node_id,kind,data) VALUES($1,$2,$3,$4) \
             ON CONFLICT(space_id,structure_node_id) DO UPDATE SET data=excluded.data,kind=excluded.kind \
             WHERE spaces_structure_layouts.data<>excluded.data OR spaces_structure_layouts.kind<>excluded.kind",
        )
        .bind(space_id)
        .bind(layout.structure_node_id())
        .bind(layout.kind_name())
        .bind(data)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    sqlx::query("DELETE FROM spaces_relation_styles WHERE space_id=$1 AND relation_id NOT IN (SELECT value FROM json_each($2))")
        .bind(space_id)
        .bind(serde_json::to_string(&snapshot.relation_styles.iter().map(|style| &style.relation_id).collect::<Vec<_>>()).map_err(db::json_error)?)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    for style in &snapshot.relation_styles {
        sqlx::query(
            "INSERT INTO spaces_relation_styles(space_id,relation_id,color) VALUES($1,$2,$3) \
             ON CONFLICT(space_id,relation_id) DO UPDATE SET color=excluded.color \
             WHERE spaces_relation_styles.color<>excluded.color",
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
        let structure_write = sqlx::query(
            "DELETE FROM spaces_structure_nodes WHERE space_id=$1 AND id=$2 AND deleted_at IS NULL",
        )
        .bind(space_id)
        .bind(structure_node_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if structure_write.rows_affected() != 1 {
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
) -> Result<HashMap<String, (StructureNode, bool)>, String> {
    let rows = sqlx::query(
        "SELECT id,kind,node_id,parent_id,order_index,title,locked,deleted_at FROM spaces_structure_nodes WHERE space_id=$1",
    )
    .bind(space_id)
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
    let structure_write = sqlx::query(
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
    if structure_write.rows_affected() != 1 {
        return Err(format!(
            "Structure node ID belongs to another Space: {}",
            structure_node.id()
        ));
    }
    Ok(())
}

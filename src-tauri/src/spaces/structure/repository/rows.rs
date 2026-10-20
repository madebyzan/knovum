//! Row-scoped structure persistence.
//!
//! A whole-snapshot save and an undo write the same tables under the same constraints — sibling
//! order is unique among live rows, a layout requires its structure row, a style requires its
//! relation — so both go through these writers. The difference is only the set of rows: a save
//! reconciles the Space, a replay touches what one record names.

use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};

use super::super::contracts::{RelationStyle, StructureLayout, StructureNode};
use crate::spaces::db;

/// A structure row and the layout that belongs to it: the two are created, moved and removed
/// together, and a record of an edit stores exactly this pair.
#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructureRow {
    pub node: StructureNode,
    pub layout: StructureLayout,
}

/// Takes the named rows out of the live set. Writing a row back only after every row it could
/// collide with is gone keeps `spaces_structure_active_order` satisfied at every statement.
pub(crate) async fn vacate(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure_node_ids: &[String],
) -> Result<(), String> {
    if structure_node_ids.is_empty() {
        return Ok(());
    }
    let now = crate::foundation::clock::unix_time_millis()?;
    sqlx::query(
        "UPDATE spaces_structure_nodes SET deleted_at=$1 \
         WHERE space_id=$2 AND deleted_at IS NULL AND id IN (SELECT value FROM json_each($3))",
    )
    .bind(now)
    .bind(space_id)
    .bind(sqlx::types::Json(structure_node_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

/// Removes rows that leave the Space for good rather than into its Trash: a subtree that moved to
/// another Space keeps its identity there, so the source cannot hold a copy of it. Deepest first,
/// because a parent cannot go before its children.
pub(crate) async fn erase(
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
        let erased = sqlx::query("DELETE FROM spaces_structure_nodes WHERE space_id=$1 AND id=$2")
            .bind(space_id)
            .bind(structure_node_id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
        if erased.rows_affected() != 1 {
            return Err(format!(
                "Structure node changed during transaction: {structure_node_id}"
            ));
        }
    }
    Ok(())
}

pub(crate) async fn write(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    rows: &[StructureRow],
) -> Result<(), String> {
    for row in rows {
        upsert_structure_node(tx, space_id, &row.node).await?;
    }
    write_layouts(
        tx,
        space_id,
        &rows.iter().map(|row| &row.layout).collect::<Vec<_>>(),
    )
    .await
}

pub(crate) async fn write_layouts(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    layouts: &[&StructureLayout],
) -> Result<(), String> {
    if layouts.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO spaces_structure_layouts(space_id,structure_node_id,kind,data) \
         SELECT $1,json_extract(value,'$.structureNodeId'),json_extract(value,'$.kind'),value \
         FROM json_each($2) WHERE 1 \
         ON CONFLICT(space_id,structure_node_id) DO UPDATE SET data=excluded.data,kind=excluded.kind \
         WHERE spaces_structure_layouts.data<>excluded.data OR spaces_structure_layouts.kind<>excluded.kind",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(layouts))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(crate) async fn remove_relation_styles(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    relation_ids: &[String],
) -> Result<(), String> {
    if relation_ids.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "DELETE FROM spaces_relation_styles \
         WHERE space_id=$1 AND relation_id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(relation_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(crate) async fn write_relation_styles(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    styles: &[RelationStyle],
) -> Result<(), String> {
    if styles.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO spaces_relation_styles(space_id,relation_id,color) \
         SELECT $1,json_extract(value,'$.relationId'),json_extract(value,'$.color') \
         FROM json_each($2) WHERE 1 \
         ON CONFLICT(space_id,relation_id) DO UPDATE SET color=excluded.color \
         WHERE spaces_relation_styles.color<>excluded.color",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(styles))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(super) async fn upsert_structure_node(
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

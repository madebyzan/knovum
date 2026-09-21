use std::collections::HashSet;

use sqlx::{Row, Sqlite, Transaction};

use super::super::contract::StructureNode;
use super::super::{RelationStyle, StructureLayout};
use crate::spaces::db;

pub(super) async fn structure(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<StructureNode>, String> {
    let rows = sqlx::query(
        "SELECT id,kind,node_id,parent_id,order_index,title,locked \
         FROM spaces_structure_nodes WHERE space_id=$1 AND deleted_at IS NULL \
         ORDER BY parent_id,order_index,id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(decode_structure_node).collect()
}

pub(super) async fn layouts(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<StructureLayout>, String> {
    let rows = sqlx::query(
        "SELECT l.data FROM spaces_structure_layouts l \
         JOIN spaces_structure_nodes i ON i.space_id=l.space_id AND i.id=l.structure_node_id \
         WHERE l.space_id=$1 AND i.deleted_at IS NULL ORDER BY i.parent_id,i.order_index,i.id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            let data: String = row.try_get("data").map_err(db::sql_error)?;
            serde_json::from_str(&data).map_err(db::json_error)
        })
        .collect()
}

pub(super) async fn active_node_ids(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<HashSet<String>, String> {
    let ids: Vec<String> =
        sqlx::query_scalar("SELECT id FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL")
            .bind(space_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    Ok(ids.into_iter().collect())
}

pub(super) async fn relation_styles(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<RelationStyle>, String> {
    let rows = sqlx::query(
        "SELECT json_object('relationId',s.relation_id,'color',s.color) AS data \
         FROM spaces_relation_styles s JOIN spaces_relations r \
         ON r.space_id=s.space_id AND r.id=s.relation_id \
         WHERE s.space_id=$1 ORDER BY s.relation_id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            let data: String = row.try_get("data").map_err(db::sql_error)?;
            serde_json::from_str(&data).map_err(db::json_error)
        })
        .collect()
}

pub(super) async fn validate_relation_styles(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    styles: &[RelationStyle],
) -> Result<(), String> {
    if styles.len() > 100_000 {
        return Err("Relation styles exceed 100000 entries".into());
    }
    let current: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_relations WHERE space_id=$1 AND status<>'invalidated'",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let current: HashSet<String> = current.into_iter().collect();
    let mut ids = HashSet::with_capacity(styles.len());
    for style in styles {
        if style.relation_id.is_empty()
            || style.relation_id.encode_utf16().count() > 128
            || !ids.insert(style.relation_id.as_str())
        {
            return Err(format!(
                "Invalid or duplicate relation style: {}",
                style.relation_id
            ));
        }
        if !current.contains(&style.relation_id) {
            return Err(format!(
                "Relation style references a missing relation: {}",
                style.relation_id
            ));
        }
    }
    Ok(())
}

fn decode_structure_node(row: sqlx::sqlite::SqliteRow) -> Result<StructureNode, String> {
    let id: String = row.try_get("id").map_err(db::sql_error)?;
    let kind: String = row.try_get("kind").map_err(db::sql_error)?;
    let parent_id: Option<String> = row.try_get("parent_id").map_err(db::sql_error)?;
    let order: i64 = row.try_get("order_index").map_err(db::sql_error)?;
    let order = u32::try_from(order).map_err(|_| "Invalid structure order")?;
    let locked: i64 = row.try_get("locked").map_err(db::sql_error)?;
    let locked = locked == 1;
    match kind.as_str() {
        "content" => Ok(StructureNode::Content {
            id,
            parent_id,
            order,
            locked,
            node_id: row
                .try_get::<Option<String>, _>("node_id")
                .map_err(db::sql_error)?
                .ok_or("Content structure node is missing node_id")?,
        }),
        "group" | "section" => {
            let title = row
                .try_get::<Option<String>, _>("title")
                .map_err(db::sql_error)?
                .ok_or("Container structure node is missing title")?;
            if kind == "group" {
                Ok(StructureNode::Group {
                    id,
                    parent_id,
                    order,
                    locked,
                    title,
                })
            } else {
                Ok(StructureNode::Section {
                    id,
                    parent_id,
                    order,
                    locked,
                    title,
                })
            }
        }
        _ => Err(format!("Unknown structure node kind: {kind}")),
    }
}

use serde_json::Value;
use sqlx::{sqlite::SqliteRow, Row, Sqlite, Transaction};

use super::super::knowledge;
use super::decode::decode_json;
use crate::spaces::{assets, db};

use super::package_assets::PackageAssets;

pub(super) async fn load_space(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Value, String> {
    let row = sqlx::query(
        "SELECT id, kind, title, archived_at AS archivedAt, last_opened_at AS lastOpenedAt, \
         created_at AS createdAt, updated_at AS updatedAt FROM spaces WHERE id = ? LIMIT 1",
    )
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| "Space no longer exists".to_owned())?;
    crate::sqlite::row_to_json(&row)
}

pub(super) async fn list_scenes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT id,space_id,title,viewport_x,viewport_y,viewport_zoom,order_index,focus_structure_node_ids,relation_kinds,relation_statuses,query_filter,created_at,updated_at \
         FROM spaces_scenes WHERE space_id=? ORDER BY order_index,created_at",
    ).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    rows.into_iter().map(|row| Ok(serde_json::json!({
        "id": row.try_get::<String,_>("id").map_err(db::sql_error)?,
        "spaceId": row.try_get::<String,_>("space_id").map_err(db::sql_error)?,
        "title": row.try_get::<String,_>("title").map_err(db::sql_error)?,
        "viewportX": row.try_get::<f64,_>("viewport_x").map_err(db::sql_error)?,
        "viewportY": row.try_get::<f64,_>("viewport_y").map_err(db::sql_error)?,
        "viewportZoom": row.try_get::<f64,_>("viewport_zoom").map_err(db::sql_error)?,
        "orderIndex": row.try_get::<i64,_>("order_index").map_err(db::sql_error)?,
        "focusStructureNodeIds": decode_json(&row,"focus_structure_node_ids")?, "relationKinds": decode_json(&row,"relation_kinds")?,
        "relationStatuses": decode_json(&row,"relation_statuses")?, "queryFilter": decode_json(&row,"query_filter")?,
        "createdAt": row.try_get::<i64,_>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
    }))).collect()
}

pub(super) async fn list_relations(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT c.id,c.space_id AS spaceId,c.source_id AS sourceId,c.target_id AS targetId,c.kind,c.rationale,c.status,c.status_before_invalidation AS statusBeforeInvalidation,c.confidence,\
         c.actor_kind AS actorKind,c.actor_id AS actorId,c.created_at AS createdAt,c.updated_at AS updatedAt \
         FROM spaces_relations c \
         JOIN spaces_nodes s ON s.id=c.source_id AND s.space_id=c.space_id AND s.deleted_at IS NULL \
         JOIN spaces_nodes t ON t.id=c.target_id AND t.space_id=c.space_id AND t.deleted_at IS NULL \
         WHERE c.space_id=? ORDER BY c.created_at,c.id",
    ).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    crate::sqlite::rows_to_json(rows)
}

pub(super) async fn list_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    assets: &mut PackageAssets,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT id, kind, data, created_at, updated_at \
         FROM spaces_nodes WHERE space_id = ? AND deleted_at IS NULL ORDER BY created_at",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut nodes = Vec::with_capacity(rows.len());
    for row in rows {
        nodes.push(decode_node_projection(row, assets)?);
    }
    Ok(nodes)
}

pub(super) fn package_layout_assets(
    layouts: &mut [crate::spaces::structure::StructureLayout],
    assets: &mut PackageAssets,
) -> Result<(), String> {
    for layout in layouts {
        let appearance = match layout {
            crate::spaces::structure::StructureLayout::Content { appearance, .. }
            | crate::spaces::structure::StructureLayout::Section { appearance, .. } => appearance,
            crate::spaces::structure::StructureLayout::Group { .. } => continue,
        };
        let Some(appearance) = appearance.as_mut() else {
            continue;
        };
        let Some(background) = appearance.get_mut("background") else {
            continue;
        };
        let path = background
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or("Structure background asset path must be a non-empty string")?;
        *background = Value::String(assets.reference(path));
    }
    Ok(())
}

pub(super) async fn list_chunks(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT c.id, c.space_id, c.node_id, c.origin, c.kind, c.source_revision, c.text, c.locator, c.metadata, c.created_at, c.updated_at \
         FROM spaces_current_knowledge_chunks c JOIN spaces_nodes n ON n.id = c.node_id AND n.space_id = c.space_id \
         WHERE c.space_id = ? AND c.origin <> 'node' AND n.deleted_at IS NULL ORDER BY c.created_at",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(knowledge::decode_chunk_projection)
        .collect()
}

fn decode_node_projection(row: SqliteRow, assets: &mut PackageAssets) -> Result<Value, String> {
    let raw: String = row.try_get("data").map_err(db::sql_error)?;
    let mut data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
    assets::rewrite_node_asset_paths(&mut data, |path| assets.reference(path));
    Ok(serde_json::json!({
        "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
        "kind": row.try_get::<String, _>("kind").map_err(db::sql_error)?,
        "data": data,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
    }))
}

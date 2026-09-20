use serde_json::Value;
use sqlx::{sqlite::SqliteRow, Row, Sqlite, Transaction};

use super::super::{grounded, knowledge};
use super::parse_json;
use crate::spaces::{assets, db};

use super::package_assets::PackageAssets;

pub(super) async fn space(
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


pub(super) async fn canvas_state(
    tx: &mut Transaction<'_, Sqlite>, space_id: &str, assets: &mut PackageAssets,
) -> Result<Value, String> {
    let (mut document, _) = crate::spaces::canvas_state::load_in_transaction(tx, space_id).await?;
    crate::spaces::canvas_state::validation::validate_content_references(tx, space_id, &document).await?;
    for layout in &mut document.layout.items {
        use crate::spaces::canvas_state::contract::StructureLayout;
        match layout {
            StructureLayout::Content { appearance, .. } | StructureLayout::Section { appearance, .. } => {
                if let Some(path) = appearance.as_mut().and_then(|appearance| appearance.background.as_mut()) {
                    *path = assets.reference(path);
                }
            }
            StructureLayout::Group { .. } => {}
        }
    }
    Ok(serde_json::json!({ "document": document }))
}

pub(super) async fn scenes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT id,space_id,title,viewport_x,viewport_y,viewport_zoom,order_index,focus_node_ids,relation_kinds,relation_statuses,query_filter,created_at,updated_at \
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
        "focusNodeIds": parse_json(&row,"focus_node_ids")?, "relationKinds": parse_json(&row,"relation_kinds")?,
        "relationStatuses": parse_json(&row,"relation_statuses")?, "queryFilter": parse_json(&row,"query_filter")?,
        "createdAt": row.try_get::<i64,_>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
    }))).collect()
}

pub(super) async fn relations(
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

pub(super) async fn nodes(
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
        nodes.push(node(row, assets)?);
    }
    Ok(nodes)
}

pub(super) async fn chunks(
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
    rows.into_iter().map(knowledge::chunk_row).collect()
}

pub(super) async fn threads(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT id,space_id,scope,scope_node_ids,title,created_at,updated_at FROM spaces_ai_threads WHERE space_id=$1 ORDER BY created_at",
    ).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    rows.into_iter().map(grounded::thread_row).collect()
}

pub(super) async fn messages(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT m.id,m.thread_id,m.role,m.content,m.citations,m.provider,m.model,m.created_at \
         FROM spaces_ai_messages m JOIN spaces_ai_threads t ON t.id=m.thread_id WHERE t.space_id=$1 ORDER BY m.created_at",
    ).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    rows.into_iter().map(message).collect()
}

fn node(row: SqliteRow, assets: &mut PackageAssets) -> Result<Value, String> {
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

fn message(row: SqliteRow) -> Result<Value, String> {
    let mut value = grounded::message_row(row)?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Grounded message row must serialize to an object".to_owned())?;
    if object.get("provider") == Some(&Value::Null) {
        object.remove("provider");
    }
    if object.get("model") == Some(&Value::Null) {
        object.remove("model");
    }
    Ok(value)
}

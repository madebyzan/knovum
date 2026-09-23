use sqlx::{Sqlite, Transaction};

use super::contracts::RestorePackageRequest;
use crate::spaces::{db, knowledge::contracts};

pub(super) async fn insert_scenes(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.scenes {
        validate_scene(tx, &request.space.id, row).await?;
        let focus = serde_json::to_string(&row.focus_structure_node_ids).map_err(db::json_error)?;
        let kinds = serde_json::to_string(&row.relation_kinds).map_err(db::json_error)?;
        let statuses = serde_json::to_string(&row.relation_statuses).map_err(db::json_error)?;
        let filter = serde_json::to_string(&row.query_filter).map_err(db::json_error)?;
        sqlx::query(
            "INSERT INTO spaces_scenes(id,space_id,title,viewport_x,viewport_y,viewport_zoom,thumbnail_path,order_index,focus_structure_node_ids,relation_kinds,relation_statuses,query_filter,created_at,updated_at) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.title).bind(row.viewport_x).bind(row.viewport_y).bind(row.viewport_zoom)
            .bind(Option::<String>::None).bind(row.order_index).bind(focus).bind(kinds).bind(statuses).bind(filter).bind(row.created_at).bind(row.updated_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn validate_scene(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    row: &super::contracts::PackageSceneRow,
) -> Result<(), String> {
    if row.title.trim().is_empty()
        || !row.viewport_x.is_finite()
        || !row.viewport_y.is_finite()
        || !row.viewport_zoom.is_finite()
        || !(0.25..=2.5).contains(&row.viewport_zoom)
    {
        return Err("Package Saved View title or viewport is invalid".into());
    }
    if !row.query_filter.is_object() {
        return Err("Saved View queryFilter must be an object".into());
    }
    contracts::validate_query_filter(tx, space_id, &row.query_filter).await?;
    if row
        .focus_structure_node_ids
        .iter()
        .any(|id| id.is_empty() || id.encode_utf16().count() > 128)
    {
        return Err("Package Saved View contains an invalid focus node identity".into());
    }
    for kind in &row.relation_kinds {
        contracts::validate_relation_kind(kind)?;
    }
    for status in &row.relation_statuses {
        contracts::validate_relation_status(status)?;
    }
    Ok(())
}

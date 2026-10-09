use sqlx::{Sqlite, Transaction};

use super::contracts::RestorePackageRequest;
use crate::spaces::db;

pub(super) async fn insert_scenes(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.scenes {
        validate_scene(tx, &request.space.id, row).await?;
        sqlx::query(
            "INSERT INTO spaces_scenes(id,space_id,title,viewport_x,viewport_y,viewport_zoom,thumbnail_path,order_index,focus_structure_node_ids,relation_kinds,relation_statuses,query_filter,created_at,updated_at) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.title).bind(row.viewport_x).bind(row.viewport_y).bind(row.viewport_zoom)
            .bind(Option::<String>::None).bind(row.order_index).bind(sqlx::types::Json(&row.focus_structure_node_ids)).bind(sqlx::types::Json(&row.relation_kinds)).bind(sqlx::types::Json(&row.relation_statuses)).bind(sqlx::types::Json(&row.query_filter)).bind(row.created_at).bind(row.updated_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn validate_scene(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    row: &super::contracts::PackageScene,
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
    crate::spaces::knowledge::validation::validate_query_filter(tx, space_id, &row.query_filter)
        .await?;
    if row
        .focus_structure_node_ids
        .iter()
        .any(|id| id.is_empty() || id.encode_utf16().count() > 128)
    {
        return Err("Package Saved View contains an invalid focus node identity".into());
    }
    Ok(())
}

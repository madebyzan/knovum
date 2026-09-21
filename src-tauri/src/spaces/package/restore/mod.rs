use sqlx::{Sqlite, Transaction};
use tauri::{AppHandle, Runtime};

pub(super) mod contracts;

use crate::spaces::{
    db,
    knowledge::{contracts as knowledge_contracts, types},
};
use contracts::RestorePackageRequest;
use validation::validate_request_shape;

pub async fn restore<R: Runtime>(
    app: &AppHandle<R>,
    mut request: RestorePackageRequest,
) -> Result<(), String> {
    validate_request_shape(&request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    insert_space(&mut tx, &request).await?;
    insert_object_types(&mut tx, &request).await?;
    insert_nodes(&mut tx, &request).await?;
    knowledge::insert_relations(&mut tx, &request).await?;
    let structure = crate::spaces::structure::repository::SpaceStructureSnapshot {
        structure: std::mem::take(&mut request.structure_nodes),
        layouts: std::mem::take(&mut request.layouts),
        relation_styles: std::mem::take(&mut request.relation_styles),
    };
    insert_structure(&mut tx, &request, &structure).await?;
    insert_scenes(&mut tx, &request).await?;
    validate_nodes(&mut tx, &request).await?;
    derivations::insert(&mut tx, &request).await?;
    workflow_runs::insert(&mut tx, &request).await?;
    knowledge::insert_provenance(&mut tx, &request).await?;
    content::insert_chunks(&mut tx, &request).await?;
    content::insert_threads(&mut tx, &request).await?;
    content::insert_messages(&mut tx, &request).await?;
    knowledge::insert_events(&mut tx, &request).await?;
    tx.commit().await.map_err(db::sql_error)
}

async fn insert_structure(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
    snapshot: &crate::spaces::structure::repository::SpaceStructureSnapshot,
) -> Result<(), String> {
    sqlx::query("INSERT INTO spaces_edit_state(space_id,revision,updated_at) VALUES($1,0,$2)")
        .bind(&request.space.id)
        .bind(request.space.updated_at)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    crate::spaces::structure::repository::save_in_transaction(tx, &request.space.id, 0, snapshot)
        .await?;
    Ok(())
}

mod content;
mod derivations;
mod derived_chunks;
mod knowledge;
mod validation;
mod workflow_runs;

async fn insert_space(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    let row = &request.space;
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,archived_at,last_opened_at,created_at,updated_at) \
         VALUES($1,$2,$3,NULL,$4,$5,$6)",
    )
    .bind(&row.id)
    .bind(&row.kind)
    .bind(&row.title)
    .bind(row.last_opened_at)
    .bind(row.created_at)
    .bind(row.updated_at)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

async fn insert_object_types(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.object_types {
        types::validate_custom_type_definition(
            &row.id,
            &row.name,
            &row.icon,
            &row.description,
            &row.fields,
        )?;
        let fields = serde_json::to_string(&row.fields).map_err(db::json_error)?;
        sqlx::query(
            "INSERT INTO spaces_object_types(id,space_id,name,icon,description,fields,system,created_at,updated_at) \
             VALUES($1,$2,$3,$4,$5,$6,0,$7,$8)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.name).bind(&row.icon).bind(&row.description)
            .bind(fields).bind(row.created_at).bind(row.updated_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn insert_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.nodes {
        let data_kind = row
            .data
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "Package node data.kind is required".to_string())?;
        if data_kind != row.kind {
            return Err("Package node kind does not match data.kind".into());
        }
        let object_type = if row.kind == "object" {
            Some(
                row.data
                    .get("objectTypeId")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| "Package object requires objectTypeId".to_string())?,
            )
        } else {
            None
        };
        let data = serde_json::to_string(&row.data).map_err(db::json_error)?;
        sqlx::query(
            "INSERT INTO spaces_nodes(id,space_id,kind,object_type_id,initial_x,initial_y,data,created_at,updated_at) VALUES($1,$2,$3,$4,0,0,$5,$6,$7)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.kind).bind(object_type)
            .bind(data).bind(row.created_at).bind(row.updated_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn validate_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.nodes {
        knowledge_contracts::validate_node(tx, &request.space.id, &row.id, &row.kind, &row.data)
            .await?;
    }
    Ok(())
}

async fn insert_scenes(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.scenes {
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
        knowledge_contracts::validate_query_filter(tx, &request.space.id, &row.query_filter)
            .await?;
        for id in &row.focus_structure_node_ids {
            if id.is_empty() || id.encode_utf16().count() > 128 {
                return Err("Package Saved View contains an invalid focus node identity".into());
            }
        }
        for kind in &row.relation_kinds {
            knowledge_contracts::validate_relation_kind(kind)?;
        }
        for status in &row.relation_statuses {
            knowledge_contracts::validate_relation_status(status)?;
        }
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

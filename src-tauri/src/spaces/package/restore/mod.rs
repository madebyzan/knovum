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
    runs::insert(&mut tx, &request).await?;
    assistant::insert(&mut tx, &request).await?;
    external_conversations::insert(&mut tx, &request).await?;
    knowledge::insert_provenance(&mut tx, &request).await?;
    content::insert_chunks(&mut tx, &request).await?;
    for message in &request.assistant_messages {
        let citations = serde_json::Value::Array(message.content.citations.clone());
        crate::spaces::knowledge::grounding::validate_assistant_citations(
            &mut tx,
            &request.space.id,
            &citations,
        )
        .await?;
    }
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

mod assistant;
mod content;
mod derivations;
mod derived_chunks;
mod external_conversations;
mod knowledge;
mod runs;
mod scenes;
mod validation;
use scenes::insert_scenes;

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

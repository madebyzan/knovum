use sqlx::{Sqlite, Transaction};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

pub(super) mod contracts;

use crate::{
    foundation::app_error::AppError,
    spaces::{db, knowledge::object_types},
};
use contracts::RestorePackageRequest;
use validation::validate_request_shape_cancellable;

pub async fn restore<R: Runtime>(
    app: &AppHandle<R>,
    mut request: RestorePackageRequest,
    cancellation: &CancellationToken,
) -> Result<(), AppError> {
    super::control::check(cancellation)?;
    validate_request_shape_cancellable(&request, cancellation)?;
    super::control::check(cancellation)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    insert_space(&mut tx, &request).await?;
    insert_object_types(&mut tx, &request).await?;
    super::control::check(cancellation)?;
    insert_nodes(&mut tx, &request).await?;
    knowledge::insert_relations(&mut tx, &request).await?;
    super::control::check(cancellation)?;
    let mut structure = crate::spaces::structure::repository::SpaceStructureSnapshot {
        structure: std::mem::take(&mut request.structure_nodes),
        layouts: std::mem::take(&mut request.layouts),
        relation_styles: std::mem::take(&mut request.relation_styles),
    };
    insert_structure(&mut tx, &request, &mut structure).await?;
    insert_scenes(&mut tx, &request).await?;
    validate_nodes(&mut tx, &request).await?;
    super::control::check(cancellation)?;
    derivations::insert(&mut tx, &request).await?;
    runs::insert(&mut tx, &request).await?;
    conversations::insert(&mut tx, &request).await?;
    external_conversations::insert(&mut tx, &request).await?;
    super::control::check(cancellation)?;
    speech::insert(&mut tx, &request).await?;
    knowledge::insert_provenance(&mut tx, &request).await?;
    content::insert_chunks(&mut tx, &request).await?;
    for message in &request.conversation_messages {
        super::control::check(cancellation)?;
        let citations = serde_json::Value::Array(message.content.citations.clone());
        crate::spaces::knowledge::grounding::validate_conversation_citations(
            &mut tx,
            &request.space.id,
            &citations,
        )
        .await?;
    }
    for derivation in &request.derivations {
        super::control::check(cancellation)?;
        if let Some(document) = &derivation.grounded_document {
            crate::spaces::knowledge::grounding::validate_conversation_citations(
                &mut tx,
                &request.space.id,
                &document.citations,
            )
            .await?;
        }
    }
    knowledge::insert_events(&mut tx, &request).await?;
    super::control::check(cancellation)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(())
}

async fn insert_structure(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
    snapshot: &mut crate::spaces::structure::repository::SpaceStructureSnapshot,
) -> Result<(), AppError> {
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
mod conversations;
mod derivations;
mod derived_chunks;
mod external_conversations;
mod knowledge;
mod runs;
mod scenes;
mod speech;
mod validation;
use scenes::insert_scenes;

async fn insert_space(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), AppError> {
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
) -> Result<(), AppError> {
    for row in &request.object_types {
        object_types::validate_custom_type_definition(
            &row.id,
            &row.name,
            &row.icon,
            &row.description,
            &row.fields,
        )?;
        sqlx::query(
            "INSERT INTO spaces_object_types(id,space_id,name,icon,description,fields,system,created_at,updated_at) \
             VALUES($1,$2,$3,$4,$5,$6,0,$7,$8)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.name).bind(&row.icon).bind(&row.description)
            .bind(sqlx::types::Json(&row.fields)).bind(row.created_at).bind(row.updated_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn insert_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), AppError> {
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
        sqlx::query(
            "INSERT INTO spaces_nodes(id,space_id,kind,object_type_id,initial_x,initial_y,data,created_at,updated_at) VALUES($1,$2,$3,$4,0,0,$5,$6,$7)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.kind).bind(object_type)
            .bind(sqlx::types::Json(&row.data)).bind(row.created_at).bind(row.updated_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn validate_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), AppError> {
    for row in &request.nodes {
        crate::spaces::knowledge::validation::validate_node(
            tx,
            &request.space.id,
            &row.kind,
            &row.data,
        )
        .await?;
    }
    Ok(())
}

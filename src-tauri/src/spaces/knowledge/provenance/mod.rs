use serde_json::Value;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use super::{events, locator, semantic_nodes};
use crate::{app_error::AppError, spaces::db};

mod command_contracts;
pub use command_contracts::{KnowledgeEventRequest, ProvenanceInput};

#[tauri::command]
pub async fn list_provenance<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
    target_kind: String,
    target_id: String,
) -> Result<Vec<Value>, AppError> {
    validation::validate_target_kind(&target_kind)?;
    let pool = db::pool(&app).await?;
    let rows = sqlx::query(
        "SELECT id,space_id,target_kind,target_id,source_node_id,locator,source_revision,quote,status,\
         actor_kind,actor_id,created_at,updated_at FROM spaces_provenance \
         WHERE space_id=$1 AND target_kind=$2 AND target_id=$3 ORDER BY created_at DESC",
    ).bind(space_id).bind(target_kind).bind(target_id).fetch_all(pool).await.map_err(db::sql_error)?;
    Ok((rows
        .into_iter()
        .map(provenance_row)
        .collect::<Result<_, String>>())?)
}

#[tauri::command]
pub async fn list_knowledge_events<R: Runtime>(
    app: AppHandle<R>,
    request: KnowledgeEventRequest,
) -> Result<Vec<Value>, AppError> {
    let KnowledgeEventRequest {
        space_id,
        entity_kind,
        entity_id,
        limit,
    } = request;
    if !(1..=1000).contains(&limit) {
        return Err("Knowledge event limit must be between 1 and 1000".into());
    }
    let pool = db::pool(&app).await?;
    let rows = match (entity_kind, entity_id) {
        (Some(entity_kind), Some(entity_id)) => {
            events::validation::validate_entity_kind(&entity_kind)?;
            // Entity IDs are globally stable across Space moves. Keep the Space column as event-time
            // context, but do not split one entity's history when its current Space changes.
            sqlx::query(
                "SELECT id,space_id,entity_kind,entity_id,event_kind,actor_kind,actor_id,payload,created_at \
                 FROM spaces_knowledge_events WHERE entity_kind=$1 AND entity_id=$2 \
                 ORDER BY created_at DESC,id DESC LIMIT $3",
            ).bind(entity_kind).bind(entity_id).bind(limit).fetch_all(pool).await.map_err(db::sql_error)?
        }
        (None, None) => sqlx::query(
            "SELECT id,space_id,entity_kind,entity_id,event_kind,actor_kind,actor_id,payload,created_at \
             FROM spaces_knowledge_events WHERE space_id=$1 ORDER BY created_at DESC,id DESC LIMIT $2",
        ).bind(space_id).bind(limit).fetch_all(pool).await.map_err(db::sql_error)?,
        _ => return Err("Knowledge event entity kind and ID must be provided together".into()),
    };
    Ok((rows
        .into_iter()
        .map(event_row)
        .collect::<Result<_, String>>())?)
}

#[tauri::command]
pub async fn create_provenance<R: Runtime>(
    app: AppHandle<R>,
    input: ProvenanceInput,
) -> Result<(), AppError> {
    validate_input(&input)?;
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    ensure_target(&mut tx, &input).await?;
    let superseded = sqlx::query(
        "SELECT id FROM spaces_provenance WHERE space_id=$1 AND target_kind=$2 AND target_id=$3 AND source_node_id=$4 AND status='active'",
    ).bind(&input.space_id).bind(&input.target_kind).bind(&input.target_id).bind(&input.source_node_id)
        .fetch_all(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_provenance SET status='superseded',updated_at=$1 \
         WHERE space_id=$2 AND target_kind=$3 AND target_id=$4 AND source_node_id=$5 AND status='active'",
    ).bind(now).bind(&input.space_id).bind(&input.target_kind).bind(&input.target_id)
        .bind(&input.source_node_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    for row in superseded {
        let provenance_id: String = row.try_get("id").map_err(db::sql_error)?;
        events::append(
            &mut tx,
            events::KnowledgeEvent::new(
                &input.space_id,
                events::EventEntity::new("provenance", &provenance_id),
                "superseded",
                events::EventActor::new("human", None),
                &serde_json::json!({}),
                now,
            ),
        )
        .await?;
    }
    sqlx::query(
        "INSERT INTO spaces_provenance(id,space_id,target_kind,target_id,source_node_id,locator,source_revision,\
         quote,status,actor_kind,actor_id,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'active',$9,$10,$11,$11)",
    ).bind(&input.id).bind(&input.space_id).bind(&input.target_kind).bind(&input.target_id)
        .bind(&input.source_node_id).bind(sqlx::types::Json(&input.locator)).bind(&input.source_revision).bind(&input.quote)
        .bind("human").bind(Option::<&str>::None).bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    events::append(
        &mut tx,
        events::KnowledgeEvent::new(
            &input.space_id,
            events::EventEntity::new("provenance", &input.id),
            "created",
            events::EventActor::new("human", None),
            &serde_json::json!({"targetKind": input.target_kind, "targetId": input.target_id}),
            now,
        ),
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(())
}

#[tauri::command]
pub async fn revoke_provenance<R: Runtime>(
    app: AppHandle<R>,
    provenance_id: String,
) -> Result<(), AppError> {
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let row = sqlx::query(
        "SELECT space_id FROM spaces_provenance WHERE id=$1 AND status <> 'invalidated'",
    )
    .bind(&provenance_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| "Provenance no longer exists".to_string())?;
    let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_provenance SET status='invalidated',updated_at=$1 WHERE id=$2")
        .bind(now)
        .bind(&provenance_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    events::append(
        &mut tx,
        events::KnowledgeEvent::new(
            &space_id,
            events::EventEntity::new("provenance", &provenance_id),
            "invalidated",
            events::EventActor::new("human", None),
            &serde_json::json!({}),
            now,
        ),
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(())
}

pub(crate) mod validation;
use validation::{ensure_target, validate_input};

mod rows;
use rows::{event_row, provenance_row};

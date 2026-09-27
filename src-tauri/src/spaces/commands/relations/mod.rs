use crate::command_error::CommandError;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::spaces::{
    db,
    knowledge::{contracts, events, invalidation},
};

pub(crate) mod batch;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpaceRelationInput {
    pub(crate) id: String,
    pub(crate) space_id: String,
    pub(crate) source_id: String,
    pub(crate) target_id: String,
    pub(crate) kind: String,
    pub(crate) rationale: String,
    pub(crate) status: String,
    pub(crate) confidence: Option<f64>,
}

async fn prepare_relation_write<R: Runtime>(
    app: &AppHandle<R>,
    relation: &SpaceRelationInput,
) -> Result<
    (
        sqlx::Transaction<'static, sqlx::Sqlite>,
        String,
        String,
        i64,
    ),
    String,
> {
    validate_relation(relation)?;
    let (source_id, target_id) = canonical_endpoints(relation);
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    contracts::validate_relation_endpoints(&mut tx, &relation.space_id, &source_id, &target_id)
        .await?;
    Ok((tx, source_id, target_id, now))
}

#[tauri::command]
pub async fn create_relation<R: Runtime>(
    app: AppHandle<R>,
    relation: SpaceRelationInput,
) -> Result<(), CommandError> {
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    create_relation_in_transaction(&mut tx, &relation, "human", None, now).await?;
    Ok((tx.commit().await.map_err(db::sql_error))?)
}

pub(crate) async fn create_relation_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    relation: &SpaceRelationInput,
    actor_kind: &str,
    actor_id: Option<&str>,
    now: i64,
) -> Result<(), String> {
    validate_relation(relation)?;
    let (source_id, target_id) = canonical_endpoints(relation);
    contracts::validate_relation_endpoints(tx, &relation.space_id, &source_id, &target_id).await?;
    sqlx::query(
        "INSERT INTO spaces_relations(\
           id, space_id, source_id, target_id, kind, rationale, status, confidence, actor_kind, actor_id, created_at, updated_at\
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$11)",
    )
    .bind(&relation.id).bind(&relation.space_id).bind(&source_id)
    .bind(&target_id).bind(&relation.kind).bind(&relation.rationale)
    .bind(&relation.status).bind(relation.confidence).bind(actor_kind)
    .bind(actor_id).bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    append_relation_event(
        tx,
        relation,
        actor_kind,
        actor_id,
        "created",
        serde_json::json!({"kind": relation.kind}),
        now,
    )
    .await
}

#[tauri::command]
pub async fn update_relation<R: Runtime>(
    app: AppHandle<R>,
    relation: SpaceRelationInput,
) -> Result<(), CommandError> {
    let (mut tx, source_id, target_id, now) = prepare_relation_write(&app, &relation).await?;
    let executed = sqlx::query(
        "UPDATE spaces_relations SET source_id=$1,target_id=$2,kind=$3,rationale=$4,status=$5,\
         confidence=$6,actor_kind=$7,actor_id=$8,updated_at=$9 WHERE id=$10 AND space_id=$11 AND status <> 'invalidated'",
    )
    .bind(&source_id).bind(&target_id).bind(&relation.kind)
    .bind(&relation.rationale).bind(&relation.status).bind(relation.confidence)
    .bind("human").bind(Option::<&str>::None).bind(now)
    .bind(&relation.id).bind(&relation.space_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(executed, "Space Relation no longer exists")?;
    append_relation_event(
        &mut tx,
        &relation,
        "human",
        None,
        "updated",
        serde_json::json!({"kind": relation.kind, "status": relation.status}),
        now,
    )
    .await?;
    Ok((tx.commit().await.map_err(db::sql_error))?)
}

#[tauri::command]
pub async fn revoke_relation<R: Runtime>(
    app: AppHandle<R>,
    relation_id: String,
) -> Result<(), CommandError> {
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    if !invalidation::invalidate_relation(&mut tx, &relation_id, "human", None, now).await? {
        return Err(("Space Relation no longer exists".to_string()).into());
    }
    Ok((tx.commit().await.map_err(db::sql_error))?)
}

#[tauri::command]
pub async fn restore_relation<R: Runtime>(
    app: AppHandle<R>,
    relation_id: String,
) -> Result<(), CommandError> {
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    invalidation::restore_relation(&mut tx, &relation_id, "human", None, now).await?;
    Ok((tx.commit().await.map_err(db::sql_error))?)
}

pub(crate) fn validate_relation(relation: &SpaceRelationInput) -> Result<(), String> {
    contracts::validate_relation_kind(&relation.kind)?;
    contracts::validate_current_relation_status(&relation.status)?;
    contracts::validate_relation_rationale(&relation.rationale)?;
    contracts::validate_confidence(relation.confidence)
}

pub(crate) fn canonical_endpoints(relation: &SpaceRelationInput) -> (String, String) {
    contracts::canonical_relation_endpoints(
        &relation.kind,
        &relation.source_id,
        &relation.target_id,
    )
}

pub(crate) async fn append_relation_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    relation: &SpaceRelationInput,
    actor_kind: &str,
    actor_id: Option<&str>,
    event_kind: &str,
    payload: serde_json::Value,
    now: i64,
) -> Result<(), String> {
    events::append(
        tx,
        events::KnowledgeEvent::new(
            &relation.space_id,
            events::EventEntity::new("relation", &relation.id),
            event_kind,
            events::EventActor::new(actor_kind, actor_id),
            &payload,
            now,
        ),
    )
    .await
}

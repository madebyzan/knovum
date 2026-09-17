use serde::Deserialize;
use serde_json::Value;
use sqlx::Row;

use tauri::{AppHandle, Runtime};

use super::{events, node_contracts};
use crate::spaces::db;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectTypeInput {
    id: String,
    space_id: String,
    name: String,
    icon: String,
    description: String,
    fields: Value,
}

#[tauri::command]
pub async fn list_object_types<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<Vec<Value>, String> {
    let pool = db::pool(&app).await?;
    let rows = sqlx::query(
        "SELECT id,space_id,name,icon,description,fields,system,created_at,updated_at \
         FROM spaces_object_types WHERE system=1 OR space_id=$1 ORDER BY system DESC,name COLLATE NOCASE",
    ).bind(space_id).fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter().map(row_to_json).collect()
}

#[tauri::command]
pub async fn create_object_type<R: Runtime>(
    app: AppHandle<R>,
    input: ObjectTypeInput,
) -> Result<(), String> {
    validate_input(&input)?;
    let now = crate::clock::unix_time_millis()?;
    let fields = serde_json::to_string(&input.fields).map_err(db::json_error)?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query(
        "INSERT INTO spaces_object_types(id,space_id,name,icon,description,fields,system,created_at,updated_at) \
         VALUES($1,$2,$3,$4,$5,$6,0,$7,$7)",
    ).bind(&input.id).bind(&input.space_id).bind(&input.name).bind(&input.icon).bind(&input.description)
        .bind(fields).bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    events::append(
        &mut tx,
        events::EventRecord::new(
            &input.space_id,
            events::EventEntity::new("type", &input.id),
            "created",
            events::EventActor::new("human", None),
            &serde_json::json!({"name": input.name}),
            now,
        ),
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)
}

#[tauri::command]
pub async fn update_object_type<R: Runtime>(
    app: AppHandle<R>,
    input: ObjectTypeInput,
) -> Result<(), String> {
    validate_input(&input)?;
    let now = crate::clock::unix_time_millis()?;
    let fields = serde_json::to_string(&input.fields).map_err(db::json_error)?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let executed = sqlx::query(
        "UPDATE spaces_object_types SET name=$1,icon=$2,description=$3,fields=$4,updated_at=$5 \
         WHERE id=$6 AND space_id=$7 AND system=0",
    )
    .bind(&input.name)
    .bind(&input.icon)
    .bind(&input.description)
    .bind(fields)
    .bind(now)
    .bind(&input.id)
    .bind(&input.space_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(executed, "Custom Knowledge Type no longer exists")?;
    validate_existing_objects(&mut tx, &input.space_id, &input.id).await?;
    events::append(
        &mut tx,
        events::EventRecord::new(
            &input.space_id,
            events::EventEntity::new("type", &input.id),
            "updated",
            events::EventActor::new("human", None),
            &serde_json::json!({"name": input.name}),
            now,
        ),
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)
}

#[tauri::command]
pub async fn delete_object_type<R: Runtime>(
    app: AppHandle<R>,
    object_type_id: String,
    space_id: String,
) -> Result<(), String> {
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces_nodes WHERE object_type_id=$1")
            .bind(&object_type_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    if count > 0 {
        return Err("Knowledge Type is still used by objects".into());
    }
    sqlx::query("DELETE FROM spaces_knowledge_events WHERE entity_kind='type' AND entity_id=$1")
        .bind(&object_type_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    let executed =
        sqlx::query("DELETE FROM spaces_object_types WHERE id=$1 AND space_id=$2 AND system=0")
            .bind(&object_type_id)
            .bind(&space_id)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    db::require_one(executed, "Custom Knowledge Type no longer exists")?;
    tx.commit().await.map_err(db::sql_error)
}

async fn validate_existing_objects(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    object_type_id: &str,
) -> Result<(), String> {
    let rows =
        sqlx::query("SELECT id,data FROM spaces_nodes WHERE space_id=$1 AND object_type_id=$2")
            .bind(space_id)
            .bind(object_type_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    for row in rows {
        let node_id: String = row.try_get("id").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        node_contracts::validate_node(tx, space_id, &node_id, "object", &data).await?;
    }
    Ok(())
}

mod rows;
mod validation;
use rows::row_to_json;
pub(crate) use validation::validate_custom_type_definition;
use validation::validate_input;

use serde_json::{Map, Value};
use sqlx::{Sqlite, Transaction};

use crate::spaces::{
    db,
    knowledge::chunk_store,
    knowledge::{contracts, events},
};

pub(in crate::spaces::knowledge::change_commit) struct ObjectInsert<'a> {
    pub space_id: &'a str,
    pub id: &'a str,
    pub object_type_id: &'a str,
    pub initial_x: f64,
    pub initial_y: f64,
    pub data: &'a Value,
    pub now: i64,
}

pub(in crate::spaces::knowledge::change_commit) async fn insert_object(
    tx: &mut Transaction<'_, Sqlite>,
    input: ObjectInsert<'_>,
) -> Result<(), String> {
    let data = serde_json::to_string(input.data).map_err(db::json_error)?;
    sqlx::query(
        "INSERT INTO spaces_nodes(id,space_id,kind,object_type_id,initial_x,initial_y,data,created_at,updated_at) \
         VALUES($1,$2,'object',$3,$4,$5,$6,$7,$7)",
    )
    .bind(input.id)
    .bind(input.space_id)
    .bind(input.object_type_id)
    .bind(input.initial_x)
    .bind(input.initial_y)
    .bind(data)
    .bind(input.now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(in crate::spaces::knowledge::change_commit) struct ObjectUpdate<'a> {
    pub space_id: &'a str,
    pub id: &'a str,
    pub title: &'a Option<String>,
    pub summary: &'a Option<String>,
    pub status: &'a Option<String>,
    pub properties: &'a Option<Map<String, Value>>,
    pub agent_id: &'a str,
    pub now: i64,
}

pub(in crate::spaces::knowledge::change_commit) async fn update_object(
    tx: &mut Transaction<'_, Sqlite>,
    input: ObjectUpdate<'_>,
) -> Result<(), String> {
    let raw = load_object_content(tx, input.space_id, input.id).await?;
    let mut data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
    let before = data.clone();
    apply_object_changes(&mut data, &input)?;
    crate::spaces::knowledge::indexing::revision::apply(&mut data)?;
    let object_type_id = contracts::validate_node(tx, input.space_id, input.id, "object", &data)
        .await?
        .ok_or_else(|| "Knowledge object type validation failed".to_string())?;
    if before.get("knowledgeRevision") != data.get("knowledgeRevision") {
        chunk_store::detach_canonical(tx, input.space_id, input.id).await?;
    }
    persist_object_update(tx, &input, &object_type_id, &data).await?;
    emit_object_updated(tx, &input, &before, &data).await
}

async fn load_object_content(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    id: &str,
) -> Result<String, String> {
    sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='object' AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| "Knowledge Change Set update target is not an active object".to_string())
}

fn apply_object_changes(data: &mut Value, input: &ObjectUpdate<'_>) -> Result<(), String> {
    let object = data
        .as_object_mut()
        .ok_or_else(|| "Knowledge object data is invalid".to_string())?;
    if let Some(value) = input.title {
        object.insert("title".into(), Value::String(value.clone()));
    }
    if let Some(value) = input.summary {
        object.insert("summary".into(), Value::String(value.clone()));
    }
    if let Some(value) = input.status {
        object.insert("status".into(), Value::String(value.clone()));
    }
    if let Some(value) = input.properties {
        object.insert("properties".into(), Value::Object(value.clone()));
    }
    Ok(())
}

async fn persist_object_update(
    tx: &mut Transaction<'_, Sqlite>,
    input: &ObjectUpdate<'_>,
    object_type_id: &str,
    data: &Value,
) -> Result<(), String> {
    let serialized = serde_json::to_string(data).map_err(db::json_error)?;
    sqlx::query(
        "UPDATE spaces_nodes SET object_type_id=$1,data=$2,updated_at=$3 WHERE id=$4 AND space_id=$5",
    )
    .bind(object_type_id)
    .bind(serialized)
    .bind(input.now)
    .bind(input.id)
    .bind(input.space_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

async fn emit_object_updated(
    tx: &mut Transaction<'_, Sqlite>,
    input: &ObjectUpdate<'_>,
    before: &Value,
    data: &Value,
) -> Result<(), String> {
    let payload = serde_json::json!({"changeSet": true});
    events::append(
        tx,
        events::KnowledgeEvent::new(
            input.space_id,
            events::EventEntity::new("node", input.id),
            "updated",
            events::EventActor::new("agent", Some(input.agent_id)),
            &payload,
            input.now,
        ),
    )
    .await?;
    events::append_object_status_transition(
        tx,
        input.space_id,
        input.id,
        before,
        data,
        events::EventActor::new("agent", Some(input.agent_id)),
        input.now,
    )
    .await
}

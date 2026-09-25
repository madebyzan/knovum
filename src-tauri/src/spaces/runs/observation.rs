use serde::Serialize;
use serde_json::Value;
use sqlx::{Row, SqlitePool};
use ts_rs::TS;

use super::{RunEventKind, StepState, WaitReason};
use crate::spaces::db;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "RunStep.ts")]
#[serde(rename_all = "camelCase")]
pub struct StepSummary {
    pub id: String,
    pub run_id: String,
    pub definition_node_id: String,
    pub instance_path: String,
    pub node_kind: String,
    pub state: StepState,
    pub state_version: i64,
    pub resolved_input: Value,
    pub output: Option<Value>,
    pub wait_reason: Option<WaitReason>,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "RunEvent.ts")]
#[serde(rename_all = "camelCase")]
pub struct RunEvent {
    pub sequence: i64,
    pub run_id: String,
    pub step_instance_id: Option<String>,
    pub attempt_id: Option<String>,
    pub kind: RunEventKind,
    pub payload: Value,
    pub created_at: i64,
}

pub async fn list_steps(pool: &SqlitePool, run_id: &str) -> Result<Vec<StepSummary>, String> {
    let rows = sqlx::query(
        "SELECT id,run_id,definition_node_id,instance_path,node_kind,state,
        state_version,resolved_input,output,wait_reason,updated_at FROM spaces_run_steps
        WHERE run_id=$1 ORDER BY created_at,id",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.iter().map(parse_step).collect()
}

pub async fn list_events(
    pool: &SqlitePool,
    run_id: &str,
    after_sequence: i64,
) -> Result<Vec<RunEvent>, String> {
    let rows = sqlx::query(
        "SELECT sequence,run_id,step_instance_id,attempt_id,kind,payload,created_at
        FROM spaces_run_events WHERE run_id=$1 AND sequence>$2 ORDER BY sequence LIMIT 1000",
    )
    .bind(run_id)
    .bind(after_sequence)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.iter().map(parse_event).collect()
}

pub async fn diagnostic_bundle(pool: &SqlitePool, run_id: &str) -> Result<Value, String> {
    let run = super::repository::load_run(pool, run_id).await?;
    let steps = list_steps(pool, run_id).await?;
    let events = list_events(pool, run_id, 0).await?;
    Ok(serde_json::json!({
        "schemaVersion": 1,
        "redaction": "content_digests_only",
        "run": run,
        "steps": steps.into_iter().map(|step| serde_json::json!({
            "id": step.id,
            "definitionNodeId": step.definition_node_id,
            "instancePath": step.instance_path,
            "nodeKind": step.node_kind,
            "state": step.state,
            "stateVersion": step.state_version,
            "waitReason": step.wait_reason,
            "resolvedInput": value_fingerprint(&step.resolved_input),
            "output": step.output.as_ref().map(value_fingerprint),
            "updatedAt": step.updated_at,
        })).collect::<Vec<_>>(),
        "events": events.into_iter().map(|event| serde_json::json!({
            "sequence": event.sequence,
            "stepInstanceId": event.step_instance_id,
            "attemptId": event.attempt_id,
            "kind": event.kind,
            "payload": value_fingerprint(&event.payload),
            "createdAt": event.created_at,
        })).collect::<Vec<_>>(),
    }))
}

fn value_fingerprint(value: &Value) -> Value {
    use sha2::Digest;
    let encoded = serde_json::to_vec(value).expect("serde_json::Value always serializes to JSON");
    serde_json::json!({
        "jsonType": match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        },
        "bytes": encoded.len(),
        "sha256": format!("{:x}", sha2::Sha256::digest(encoded)),
    })
}

fn parse_step(row: &sqlx::sqlite::SqliteRow) -> Result<StepSummary, String> {
    Ok(StepSummary {
        id: get(row, "id")?,
        run_id: get(row, "run_id")?,
        definition_node_id: get(row, "definition_node_id")?,
        instance_path: get(row, "instance_path")?,
        node_kind: get(row, "node_kind")?,
        state: parse_enum(row, "state")?,
        state_version: get(row, "state_version")?,
        resolved_input: parse_json(row, "resolved_input")?,
        output: parse_optional_json(row, "output")?,
        wait_reason: parse_optional_enum(row, "wait_reason")?,
        updated_at: get(row, "updated_at")?,
    })
}

fn parse_event(row: &sqlx::sqlite::SqliteRow) -> Result<RunEvent, String> {
    Ok(RunEvent {
        sequence: get(row, "sequence")?,
        run_id: get(row, "run_id")?,
        step_instance_id: get(row, "step_instance_id")?,
        attempt_id: get(row, "attempt_id")?,
        kind: parse_enum(row, "kind")?,
        payload: parse_json(row, "payload")?,
        created_at: get(row, "created_at")?,
    })
}

fn get<T>(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<T, String>
where
    T: for<'row> sqlx::Decode<'row, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite>,
{
    row.try_get(field).map_err(db::sql_error)
}

fn parse_json(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<Value, String> {
    serde_json::from_str(&get::<String>(row, field)?).map_err(db::json_error)
}

fn parse_optional_json(
    row: &sqlx::sqlite::SqliteRow,
    field: &str,
) -> Result<Option<Value>, String> {
    get::<Option<String>>(row, field)?
        .map(|value| serde_json::from_str(&value).map_err(db::json_error))
        .transpose()
}

fn parse_enum<T: serde::de::DeserializeOwned>(
    row: &sqlx::sqlite::SqliteRow,
    field: &str,
) -> Result<T, String> {
    serde_json::from_value(Value::String(get(row, field)?)).map_err(db::json_error)
}

fn parse_optional_enum<T: serde::de::DeserializeOwned>(
    row: &sqlx::sqlite::SqliteRow,
    field: &str,
) -> Result<Option<T>, String> {
    get::<Option<String>>(row, field)?
        .map(|value| serde_json::from_value(Value::String(value)).map_err(db::json_error))
        .transpose()
}

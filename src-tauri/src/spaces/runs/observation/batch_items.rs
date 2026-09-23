use serde::Serialize;
use serde_json::Value;
use sqlx::{Row, SqlitePool};
use ts_rs::TS;

use crate::spaces::db;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "RunBatchItem.ts")]
#[serde(rename_all = "camelCase")]
pub struct BatchItemSummary {
    pub definition_node_id: String,
    pub item_key: String,
    pub latest_run_id: String,
    pub latest_step_instance_id: String,
    pub state: BatchItemState,
    pub attempt_count: i64,
    pub input: Value,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub child_run_id: Option<String>,
    pub updated_at: i64,
}

#[derive(Clone, Copy, Debug, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum BatchItemState {
    Failed,
    Running,
    Succeeded,
    Waiting,
}

impl BatchItemState {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "failed" => Ok(Self::Failed),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "waiting" => Ok(Self::Waiting),
            _ => Err(format!("run-batch-item-invalid: unsupported state {value}")),
        }
    }
}

pub async fn list(pool: &SqlitePool, run_id: &str) -> Result<Vec<BatchItemSummary>, String> {
    let lineage: String = sqlx::query_scalar(
        "SELECT COALESCE(json_extract(policy_snapshot,'$.retryRootRunId'),id)
         FROM spaces_runs WHERE id=$1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("run-unavailable: Run is unavailable")?;
    let rows = sqlx::query(
        "SELECT definition_node_id,item_key,latest_run_id,latest_step_instance_id,state,
         attempt_count,input,output,error,child_run_id,updated_at
         FROM spaces_run_batch_items WHERE lineage_run_id=$1
         ORDER BY definition_node_id,item_key",
    )
    .bind(lineage)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.iter().map(parse).collect()
}

fn parse(row: &sqlx::sqlite::SqliteRow) -> Result<BatchItemSummary, String> {
    let input: String = row.try_get("input").map_err(db::sql_error)?;
    let output: Option<String> = row.try_get("output").map_err(db::sql_error)?;
    let state: String = row.try_get("state").map_err(db::sql_error)?;
    Ok(BatchItemSummary {
        definition_node_id: row.try_get("definition_node_id").map_err(db::sql_error)?,
        item_key: row.try_get("item_key").map_err(db::sql_error)?,
        latest_run_id: row.try_get("latest_run_id").map_err(db::sql_error)?,
        latest_step_instance_id: row
            .try_get("latest_step_instance_id")
            .map_err(db::sql_error)?,
        state: BatchItemState::parse(&state)?,
        attempt_count: row.try_get("attempt_count").map_err(db::sql_error)?,
        input: serde_json::from_str(&input).map_err(db::json_error)?,
        output: output
            .map(|value| serde_json::from_str(&value).map_err(db::json_error))
            .transpose()?,
        error: row.try_get("error").map_err(db::sql_error)?,
        child_run_id: row.try_get("child_run_id").map_err(db::sql_error)?,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
    })
}

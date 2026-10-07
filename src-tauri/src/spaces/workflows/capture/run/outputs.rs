use serde::Serialize;
use serde_json::Value;
use sqlx::{Row, SqlitePool};

use super::super::contracts::CaptureCandidate;
use crate::spaces::db;

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct CapturedRunOutput {
    pub output_slot_id: String,
    pub name: String,
    pub node_id: String,
}

pub(crate) async fn list(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Vec<CapturedRunOutput>, String> {
    let row = sqlx::query(
        "SELECT home_space_id,policy_snapshot FROM spaces_runs WHERE id=$1 AND kind='workflow'",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("workflow-run-unavailable: Run is unavailable")?;
    let space_id: String = row.try_get("home_space_id").map_err(db::sql_error)?;
    let policy: String = row.try_get("policy_snapshot").map_err(db::sql_error)?;
    let policy: Value = serde_json::from_str(&policy).map_err(db::json_error)?;
    if policy.get("captureSchemaVersion").and_then(Value::as_u64) != Some(1) {
        return Err("workflow-run-obsolete: Run uses an older capture definition".into());
    }
    let definition: CaptureCandidate = serde_json::from_value(
        policy
            .get("workflowDefinition")
            .cloned()
            .ok_or("workflow-run-invalid: definition is missing")?,
    )
    .map_err(db::json_error)?;
    let mut outputs = Vec::new();
    for slot in definition.output_slots {
        let step_output: Option<String> = sqlx::query_scalar(
            "SELECT output FROM spaces_run_steps WHERE run_id=$1 AND definition_node_id=$2 AND state='succeeded'",
        )
        .bind(run_id)
        .bind(&slot.step_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?;
        let Some(step_output) = step_output else {
            continue;
        };
        let value: Value = serde_json::from_str(&step_output).map_err(db::json_error)?;
        let child_run_id = value
            .get("childRunId")
            .and_then(Value::as_str)
            .ok_or("workflow-run-invalid: output step has no child Run")?;
        let node_id: String = sqlx::query_scalar(
            "SELECT d.output_node_id FROM spaces_knowledge_derivations d
             JOIN spaces_nodes n ON n.id=d.output_node_id AND n.space_id=d.space_id
             WHERE d.id=$1 AND d.space_id=$2 AND d.status='completed' AND n.deleted_at IS NULL",
        )
        .bind(child_run_id)
        .bind(&space_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?
        .ok_or("workflow-run-invalid: completed output is unavailable")?;
        outputs.push(CapturedRunOutput {
            output_slot_id: slot.id,
            name: slot.name,
            node_id,
        });
    }
    Ok(outputs)
}

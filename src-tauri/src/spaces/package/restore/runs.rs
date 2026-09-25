use sqlx::{Sqlite, Transaction};

use super::contracts::RestorePackageRequest;
use crate::spaces::db;

pub(super) async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.runs {
        if !matches!(
            row.state.as_str(),
            "succeeded" | "completed_with_errors" | "failed" | "cancelled"
        ) {
            return Err("Package contains a non-terminal Run".into());
        }
        sqlx::query("INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,state_version,execution_epoch,policy_snapshot,input_snapshot,budget,input_tokens,output_tokens,total_tokens,step_attempts,expanded_steps,output_bytes,created_at,updated_at,completed_at) VALUES($1,$2,$3,$4,$5,$6,$7,0,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)")
            .bind(&row.id).bind(&request.space.id).bind(&row.principal_kind).bind(&row.principal_id)
            .bind(&row.kind).bind(&row.state).bind(row.state_version).bind(json(&row.policy_snapshot)?)
            .bind(json(&row.input_snapshot)?).bind(json(&row.budget)?).bind(row.input_tokens)
            .bind(row.output_tokens).bind(row.total_tokens).bind(row.step_attempts)
            .bind(row.expanded_steps).bind(row.output_bytes)
            .bind(row.created_at).bind(row.updated_at)
            .bind(row.completed_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.runs {
        sqlx::query("UPDATE spaces_runs SET parent_run_id=$1,retry_of_run_id=$2,fork_of_run_id=$3 WHERE id=$4")
            .bind(&row.parent_run_id).bind(&row.retry_of_run_id).bind(&row.fork_of_run_id).bind(&row.id)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.run_steps {
        sqlx::query("INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,depends_on,configuration,state,state_version,resolved_input,output,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)")
            .bind(&row.id).bind(&row.run_id).bind(&row.definition_node_id).bind(&row.instance_path)
            .bind(&row.node_kind).bind(json(&row.depends_on)?).bind(json(&row.configuration)?)
            .bind(&row.state).bind(row.state_version).bind(json(&row.resolved_input)?)
            .bind(optional_json(row.output.as_ref())?).bind(row.created_at).bind(row.updated_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    insert_attempts(tx, request).await?;
    insert_events(tx, request).await
}

async fn insert_attempts(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.run_attempts {
        sqlx::query("INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,request_digest,executor_id,idempotency_key,external_task_id,execution_epoch,checkpoint,result,error_code,started_at,finished_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,0,$10,$11,$12,$13,$14)")
            .bind(&row.id).bind(&row.run_id).bind(&row.step_instance_id).bind(row.attempt_number)
            .bind(&row.state).bind(&row.request_digest).bind(format!("package:{}", row.id))
            .bind(&row.idempotency_key).bind(&row.external_task_id).bind(optional_json(row.checkpoint.as_ref())?)
            .bind(optional_json(row.result.as_ref())?).bind(&row.error_code).bind(row.started_at)
            .bind(row.finished_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn insert_events(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.run_events {
        sqlx::query("INSERT INTO spaces_run_events(run_id,step_instance_id,attempt_id,kind,payload,created_at) VALUES($1,$2,$3,$4,$5,$6)")
            .bind(&row.run_id).bind(&row.step_instance_id).bind(&row.attempt_id)
            .bind(row.kind.as_str())
            .bind(json(&row.payload)?).bind(row.created_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

fn json(value: &serde_json::Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(db::json_error)
}

fn optional_json(value: Option<&serde_json::Value>) -> Result<Option<String>, String> {
    value.map(json).transpose()
}

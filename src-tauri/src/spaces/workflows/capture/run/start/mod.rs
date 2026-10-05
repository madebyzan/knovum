use std::collections::HashSet;

use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use super::{bindings::bind_inputs, validation, StartCapturedWorkflowRequest};
use crate::spaces::{
    authorization::{create_grant, AuthorizationContext, Principal, PrincipalKind},
    db,
    runs::{create_run_in_transaction, NewRun, RunKind, RunObservation},
    workflows::capture::{contracts::CandidateSource, library::SavedCapturedWorkflow},
};

pub(crate) async fn start(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    request: StartCapturedWorkflowRequest,
) -> Result<RunObservation, String> {
    uuid::Uuid::parse_str(&request.request_id)
        .map_err(|_| "workflow-run-invalid: request ID must be a UUID")?;
    let mut tx = db::begin_write(pool).await?;
    let existing = sqlx::query("SELECT home_space_id,policy_snapshot FROM spaces_runs WHERE id=$1")
        .bind(&request.request_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    if let Some(existing) = existing {
        let home_space_id: String = existing.try_get("home_space_id").map_err(db::sql_error)?;
        let policy: String = existing.try_get("policy_snapshot").map_err(db::sql_error)?;
        let policy: serde_json::Value = serde_json::from_str(&policy).map_err(db::json_error)?;
        if home_space_id != request.space_id
            || policy
                .get("requestWorkflowId")
                .and_then(serde_json::Value::as_str)
                != Some(request.workflow_id.as_str())
            || policy.get("requestBindings") != Some(&json!(request.bindings))
            || policy.get("requestWorkflowVersion") != Some(&json!(request.expected_version))
        {
            return Err("workflow-run-conflict: request ID belongs to another Run".into());
        }
        tx.commit().await.map_err(db::sql_error)?;
        return crate::spaces::runs::load_run(pool, &request.request_id).await;
    }
    let definition: String = sqlx::query_scalar(
        "SELECT definition FROM spaces_workflow_templates WHERE id=$1
         AND json_extract(definition,'$.schemaVersion')=3",
    )
    .bind(&request.workflow_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("workflow-unavailable: saved workflow is unavailable")?;
    let saved: SavedCapturedWorkflow = serde_json::from_str(&definition).map_err(db::json_error)?;
    if saved.version != request.expected_version {
        return Err("workflow-change-conflict: reload the workflow library".into());
    }
    validation::candidate(&saved.candidate)?;
    validation::targets(app, &saved.candidate)?;
    create(&mut tx, &request, &saved).await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    crate::spaces::runs::load_run(pool, &request.request_id).await
}

async fn create(
    tx: &mut Transaction<'_, Sqlite>,
    request: &StartCapturedWorkflowRequest,
    saved: &SavedCapturedWorkflow,
) -> Result<(), String> {
    let bindings = bind_inputs(
        tx,
        &request.space_id,
        &saved.candidate.input_slots,
        &request.bindings,
    )
    .await?;
    let now = crate::clock::unix_time_millis()?;
    let run_id = request.request_id.clone();
    let grant = AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::User,
            id: "local-user".into(),
        },
        grant_id: uuid::Uuid::new_v4().to_string(),
        home_space_id: request.space_id.clone(),
        read_space_ids: vec![request.space_id.clone()],
        write_space_ids: vec![request.space_id.clone()],
        action_ids: saved
            .candidate
            .steps
            .iter()
            .map(|step| step.action_id.clone())
            .collect(),
        valid_until: now + 7 * 24 * 60 * 60 * 1_000,
    };
    create_grant(tx, &grant, now).await?;
    let definition = serde_json::to_value(&saved.candidate).map_err(db::json_error)?;
    let digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&definition).map_err(db::json_error)?)
    );
    let policy = json!({
        "grantId": grant.grant_id,
        "workflowId": saved.workflow_id,
        "requestWorkflowId": request.workflow_id,
        "requestWorkflowVersion": request.expected_version,
        "requestBindings": request.bindings,
        "workflowVersion": saved.version,
        "captureSchemaVersion": 3,
        "definitionDigest": digest,
        "workflowDefinition": definition,
        "definition": {"failurePolicy": "stop_run"},
        "bindings": bindings,
    });
    let input = json!({"bindings": bindings});
    let budget = json!({"maxParallelism": 2, "maxStepAttempts": 128});
    create_run_in_transaction(
        tx,
        &NewRun {
            id: &run_id,
            home_space_id: &request.space_id,
            parent_run_id: None,
            principal: &grant.principal,
            kind: RunKind::Workflow,
            policy_snapshot: &policy,
            input_snapshot: &input,
            budget: &budget,
        },
        now,
    )
    .await?;
    for step in &saved.candidate.steps {
        let mut seen = HashSet::new();
        let dependencies = step
            .inputs
            .iter()
            .filter_map(|input| match &input.source {
                CandidateSource::StepOutput { step_id } => Some(step_id.clone()),
                CandidateSource::InputSlot { .. } | CandidateSource::InputSequence { .. } => None,
            })
            .filter(|step_id| seen.insert(step_id.clone()))
            .collect::<Vec<_>>();
        insert_step(tx, &run_id, step, &dependencies, now).await?;
    }
    sqlx::query("UPDATE spaces_runs SET expanded_steps=$1 WHERE id=$2")
        .bind(saved.candidate.steps.len() as i64)
        .bind(&run_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

async fn insert_step(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    step: &super::super::contracts::CandidateStep,
    dependencies: &[String],
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,
         depends_on,configuration,state,state_version,resolved_input,created_at,updated_at)
         VALUES($1,$2,$3,$3,'captured_action',$4,$5,$6,0,'{}',$7,$7)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(run_id)
    .bind(&step.id)
    .bind(serde_json::to_string(dependencies).map_err(db::json_error)?)
    .bind(serde_json::to_string(step).map_err(db::json_error)?)
    .bind(if dependencies.is_empty() {
        "ready"
    } else {
        "pending"
    })
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

#[cfg(test)]
mod tests;

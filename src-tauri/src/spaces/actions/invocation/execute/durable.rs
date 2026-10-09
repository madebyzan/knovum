use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::{ActionInvocationOutcome, ActionInvocationRequest};
use crate::spaces::{
    authorization::AuthorizationContext,
    db,
    runs::{create_run_in_transaction, NewRun, RunKind},
};

pub(super) async fn create(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ActionInvocationRequest,
    authorization: &AuthorizationContext,
    now: i64,
) -> Result<ActionInvocationOutcome, String> {
    let run_id = uuid::Uuid::new_v4().to_string();
    let input = serde_json::json!({
        "actionId": request.action_id, "actionVersion": request.action_version,
        "parameters": request.parameters, "inputs": request.input_snapshot,
        "expectedRevisions": request.expected_revisions
    });
    let policy = serde_json::json!({"grantId": request.grant_id});
    let budget = serde_json::json!({"maxAttempts": 3});
    create_run_in_transaction(
        tx,
        &NewRun {
            id: &run_id,
            home_space_id: &request.space_id,
            parent_run_id: request.parent_run_id.as_deref(),
            principal: &authorization.principal,
            kind: RunKind::Action,
            policy_snapshot: &policy,
            input_snapshot: &input,
            budget: &budget,
        },
        now,
    )
    .await?;
    insert_step(tx, &run_id, &input, now).await?;
    Ok(ActionInvocationOutcome::AcceptedRun { run_id })
}

async fn insert_step(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    input: &Value,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,
         depends_on,configuration,state,state_version,resolved_input,created_at,updated_at)
         VALUES($1,$2,'action','action','action','[]','{}','ready',0,$3,$4,$4)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(run_id)
    .bind(sqlx::types::Json(input))
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

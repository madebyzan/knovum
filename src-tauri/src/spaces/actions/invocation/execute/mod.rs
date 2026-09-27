use sqlx::{Sqlite, SqlitePool, Transaction};

mod approval;
mod durable;

use super::{digest::request_digest, store, ActionInvocationOutcome, ActionInvocationRequest};
use crate::spaces::{
    actions::{SpaceActionExecutionMode, SpaceActionImpact},
    authorization::load_grant,
    db,
};

pub async fn invoke(
    pool: &SqlitePool,
    request: ActionInvocationRequest,
) -> Result<ActionInvocationOutcome, String> {
    request.validate_identity()?;
    let definition = crate::spaces::actions::require(&request.action_id)?;
    validate_definition(definition, &request)?;
    let digest = request_digest(&request)?;
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let authorization = load_grant(&mut tx, &request.grant_id).await?;
    approval::authorize(&authorization, definition, &request, now)?;
    approval::require_active_parent(&mut tx, request.parent_run_id.as_deref()).await?;
    if let Some(outcome) = store::load_existing(&mut tx, &request, &digest).await? {
        if request.approval_id.is_none()
            || !matches!(outcome, ActionInvocationOutcome::RequiresApproval { .. })
        {
            tx.commit().await.map_err(db::sql_error)?;
            return Ok(outcome);
        }
    }
    if approval::required(authorization.principal.kind, definition) {
        if let Some(outcome) = approval::handle(&mut tx, &request, &digest, now).await? {
            store::save_outcome(&mut tx, &request, &authorization, &digest, &outcome, now).await?;
            tx.commit().await.map_err(db::sql_error)?;
            return Ok(outcome);
        }
    }
    let outcome = execute(
        &mut tx,
        &request,
        &authorization,
        definition.execution_mode,
        definition.impact,
        now,
    )
    .await?;
    store::save_outcome(&mut tx, &request, &authorization, &digest, &outcome, now).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(outcome)
}

fn validate_definition(
    definition: &crate::spaces::actions::SpaceActionDefinition,
    request: &ActionInvocationRequest,
) -> Result<(), String> {
    if definition.version != request.action_version {
        return Err("action-version-unavailable: SpaceAction version is unavailable".into());
    }
    crate::spaces::actions::validate_parameters(definition, &request.parameters)
}

async fn execute(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ActionInvocationRequest,
    authorization: &crate::spaces::authorization::AuthorizationContext,
    mode: SpaceActionExecutionMode,
    impact: SpaceActionImpact,
    now: i64,
) -> Result<ActionInvocationOutcome, String> {
    match mode {
        SpaceActionExecutionMode::Immediate => {
            let action_output = crate::spaces::action_execution::execute_immediate(
                tx,
                &authorization.principal,
                &request.invocation_id,
                &request.space_id,
                &request.action_id,
                &request.parameters,
                now,
            )
            .await?;
            Ok(if impact == SpaceActionImpact::ReadOnly {
                ActionInvocationOutcome::QueryResult {
                    result: action_output,
                }
            } else {
                ActionInvocationOutcome::CommittedReceipt {
                    receipt: action_output,
                }
            })
        }
        SpaceActionExecutionMode::Durable => durable::create(tx, request, authorization, now).await,
    }
}

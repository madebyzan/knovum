use crate::app_error::AppError;
use sqlx::{Sqlite, SqlitePool, Transaction};

mod authorization;
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
) -> Result<ActionInvocationOutcome, AppError> {
    let mut tx = db::begin_write(pool).await?;
    let outcome = invoke_in_transaction(&mut tx, &request).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(outcome)
}

pub(crate) async fn invoke_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ActionInvocationRequest,
) -> Result<ActionInvocationOutcome, AppError> {
    request.validate_identity()?;
    let definition = crate::spaces::actions::require(&request.action_id)?;
    validate_definition(definition, request)?;
    let digest = request_digest(request)?;
    let now = crate::clock::unix_time_millis()?;
    let authorization = load_grant(tx, &request.grant_id).await?;
    authorization::authorize(&authorization, definition, request, now)?;
    authorization::require_active_parent(tx, request.parent_run_id.as_deref()).await?;
    if let Some(outcome) = store::load_existing(tx, request, &digest).await? {
        return Ok(outcome);
    }
    let outcome = execute(
        tx,
        request,
        &authorization,
        definition.execution_mode,
        definition.impact,
        now,
    )
    .await?;
    store::save_outcome(tx, request, &authorization, &digest, &outcome, now).await?;
    Ok(outcome)
}

fn validate_definition(
    definition: &crate::spaces::actions::SpaceActionDefinition,
    request: &ActionInvocationRequest,
) -> Result<(), AppError> {
    if definition.version != request.action_version {
        return Err("action-version-unavailable: SpaceAction version is unavailable".into());
    }
    Ok(crate::spaces::actions::validate_parameters(
        definition,
        &request.parameters,
    )?)
}

async fn execute(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ActionInvocationRequest,
    authorization: &crate::spaces::authorization::AuthorizationContext,
    mode: SpaceActionExecutionMode,
    impact: SpaceActionImpact,
    now: i64,
) -> Result<ActionInvocationOutcome, AppError> {
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
        SpaceActionExecutionMode::Durable => {
            Ok(durable::create(tx, request, authorization, now).await?)
        }
    }
}

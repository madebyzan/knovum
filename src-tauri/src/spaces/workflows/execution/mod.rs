mod action;
mod shared;

use sqlx::SqlitePool;

use crate::foundation::app_error::AppError;
use crate::spaces::runs::{AttemptClaim, StepOutcome};

pub(crate) async fn execute_captured_step(
    pool: &SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    super::capture::execute_step(pool, claim).await
}

pub(crate) async fn invoke_captured(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    action_id: String,
    action_version: u16,
    parameters: serde_json::Value,
    input: serde_json::Value,
) -> Result<StepOutcome, AppError> {
    action::execute_invocation(pool, claim, action_id, action_version, parameters, input).await
}

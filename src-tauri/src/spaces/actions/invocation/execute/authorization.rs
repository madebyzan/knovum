use sqlx::{Sqlite, Transaction};

use super::ActionInvocationRequest;
use crate::spaces::{actions::SpaceActionImpact, authorization::AuthorizationContext, db};

pub(super) async fn require_active_parent(
    tx: &mut Transaction<'_, Sqlite>,
    parent_run_id: Option<&str>,
) -> Result<(), String> {
    let Some(run_id) = parent_run_id else {
        return Ok(());
    };
    let active: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_runs WHERE id=$1 AND completed_at IS NULL \
         AND cancel_requested_at IS NULL AND pause_requested_at IS NULL",
    )
    .bind(run_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    active
        .map(|_| ())
        .ok_or_else(|| "parent-run-inactive: Run was cancelled, paused, or completed".into())
}

pub(super) fn authorize(
    context: &AuthorizationContext,
    definition: &crate::spaces::actions::SpaceActionDefinition,
    request: &ActionInvocationRequest,
    now: i64,
) -> Result<(), String> {
    match definition.impact {
        SpaceActionImpact::ReadOnly => context.require_read(&request.space_id, now),
        SpaceActionImpact::CreatesResource | SpaceActionImpact::MutatesResource => {
            context.require_action(&request.space_id, &request.action_id, now)
        }
    }
}

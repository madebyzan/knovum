use sqlx::{Sqlite, Transaction};

use super::{ActionInvocationOutcome, ActionInvocationRequest};
use crate::spaces::{
    actions::{SpaceActionApprovalPolicy, SpaceActionImpact},
    authorization::{
        consume_approval, create_approval, ApprovalRequest, AuthorizationContext, PrincipalKind,
    },
    db,
};

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

pub(super) fn required(
    principal: PrincipalKind,
    definition: &crate::spaces::actions::SpaceActionDefinition,
) -> bool {
    definition.impact != SpaceActionImpact::ReadOnly
        && definition.approval_policy == SpaceActionApprovalPolicy::ExactPlan
        && matches!(
            principal,
            PrincipalKind::Assistant | PrincipalKind::ExternalAgent
        )
}

pub(super) async fn handle(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ActionInvocationRequest,
    digest: &str,
    now: i64,
) -> Result<Option<ActionInvocationOutcome>, String> {
    if let Some(approval_id) = request.approval_id.as_deref() {
        consume_approval(tx, approval_id, &request.action_id, digest, now).await?;
        return Ok(None);
    }
    let approval_id = uuid::Uuid::new_v4().to_string();
    create_approval(
        tx,
        &ApprovalRequest {
            id: approval_id.clone(),
            grant_id: request.grant_id.clone(),
            space_id: request.space_id.clone(),
            action_id: request.action_id.clone(),
            plan_digest: digest.to_owned(),
            plan: serde_json::json!({
                "actionId": request.action_id,
                "parameters": request.parameters,
                "inputSnapshot": request.input_snapshot,
                "expectedRevisions": request.expected_revisions,
            }),
            expected_revisions: request.expected_revisions.clone(),
            valid_until: now + 15 * 60 * 1_000,
        },
        now,
    )
    .await?;
    Ok(Some(ActionInvocationOutcome::RequiresApproval {
        approval_id,
        plan_digest: digest.to_owned(),
    }))
}

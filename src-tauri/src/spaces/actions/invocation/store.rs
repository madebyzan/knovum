use sqlx::{Row, Sqlite, Transaction};

use super::{ActionInvocationOutcome, ActionInvocationRequest};
use crate::spaces::{authorization::AuthorizationContext, db};

pub(super) async fn load_existing(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ActionInvocationRequest,
    digest: &str,
) -> Result<Option<ActionInvocationOutcome>, String> {
    let row = sqlx::query(
        "SELECT request_digest,outcome FROM spaces_action_invocations
         WHERE principal_kind=(SELECT principal_kind FROM spaces_authorization_grants WHERE id=$1)
           AND principal_id=(SELECT principal_id FROM spaces_authorization_grants WHERE id=$1)
           AND space_id=$2 AND action_id=$3 AND action_version=$4 AND idempotency_key=$5",
    )
    .bind(&request.grant_id)
    .bind(&request.space_id)
    .bind(&request.action_id)
    .bind(request.action_version)
    .bind(&request.idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some(row) = row else { return Ok(None) };
    let stored_digest: String = row.try_get("request_digest").map_err(db::sql_error)?;
    if stored_digest != digest {
        return Err("idempotency-conflict: invocation parameters changed".into());
    }
    let outcome: String = row.try_get("outcome").map_err(db::sql_error)?;
    serde_json::from_str(&outcome)
        .map(Some)
        .map_err(db::json_error)
}

pub(super) async fn save_outcome(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ActionInvocationRequest,
    authorization: &AuthorizationContext,
    digest: &str,
    outcome: &ActionInvocationOutcome,
    now: i64,
) -> Result<(), String> {
    let kind = match outcome {
        ActionInvocationOutcome::QueryResult { .. } => "query_result",
        ActionInvocationOutcome::CommittedReceipt { .. } => "committed_receipt",
        ActionInvocationOutcome::AcceptedRun { .. } => "accepted_run",
    };
    sqlx::query(
        "INSERT INTO spaces_action_invocations(id,principal_kind,principal_id,space_id,
         action_id,action_version,idempotency_key,request_digest,outcome_kind,outcome,created_at)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
         ON CONFLICT(principal_kind,principal_id,space_id,action_id,action_version,idempotency_key)
         DO UPDATE SET outcome_kind=excluded.outcome_kind,outcome=excluded.outcome",
    )
    .bind(&request.invocation_id)
    .bind(authorization.principal.kind.as_str())
    .bind(&authorization.principal.id)
    .bind(&request.space_id)
    .bind(&request.action_id)
    .bind(request.action_version)
    .bind(&request.idempotency_key)
    .bind(digest)
    .bind(kind)
    .bind(serde_json::to_string(outcome).map_err(db::json_error)?)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

mod list;
pub(crate) use list::list_pending;

use serde::{Deserialize, Serialize};
use sqlx::{Row, Sqlite, Transaction};
use ts_rs::TS;

use crate::spaces::db;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/ApprovalSummary.ts")]
#[serde(rename_all = "camelCase")]
pub struct ApprovalSummary {
    pub id: String,
    pub run_id: Option<String>,
    pub action_id: String,
    pub plan_digest: String,
    pub plan: serde_json::Value,
    pub expected_revisions: serde_json::Value,
    pub valid_until: i64,
    pub created_at: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovalRequest {
    pub id: String,
    pub grant_id: String,
    pub space_id: String,
    pub action_id: String,
    pub plan_digest: String,
    pub plan: serde_json::Value,
    pub expected_revisions: serde_json::Value,
    pub valid_until: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    Approve,
    Reject,
}

pub(crate) async fn create(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ApprovalRequest,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_action_approvals(
          id,grant_id,space_id,action_id,plan_digest,plan,expected_revisions,
          valid_until,decision,decided_at,consumed_at,created_at
        ) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'pending',NULL,NULL,$9)",
    )
    .bind(&request.id)
    .bind(&request.grant_id)
    .bind(&request.space_id)
    .bind(&request.action_id)
    .bind(&request.plan_digest)
    .bind(serde_json::to_string(&request.plan).map_err(db::json_error)?)
    .bind(serde_json::to_string(&request.expected_revisions).map_err(db::json_error)?)
    .bind(request.valid_until)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(crate) async fn consume(
    tx: &mut Transaction<'_, Sqlite>,
    approval_id: &str,
    grant_id: &str,
    space_id: &str,
    action_id: &str,
    plan_digest: &str,
    now: i64,
) -> Result<(), String> {
    let row = sqlx::query(
        "SELECT grant_id,space_id,action_id,plan_digest,valid_until,decision,consumed_at
         FROM spaces_action_approvals WHERE id=$1",
    )
    .bind(approval_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("approval-required: approval is unavailable")?;
    if row
        .try_get::<Option<i64>, _>("consumed_at")
        .map_err(db::sql_error)?
        .is_some()
    {
        return Err("approval-stale: approval was already consumed".into());
    }
    let decision: String = row.try_get("decision").map_err(db::sql_error)?;
    if decision != "approved" {
        return Err(match decision.as_str() {
            "rejected" => "approval-rejected: action plan was rejected".into(),
            _ => "approval-required: action plan is awaiting approval".into(),
        });
    }
    if row
        .try_get::<String, _>("grant_id")
        .map_err(db::sql_error)?
        != grant_id
        || row
            .try_get::<String, _>("space_id")
            .map_err(db::sql_error)?
            != space_id
    {
        return Err("approval-stale: approval belongs to another authorization context".into());
    }
    if row
        .try_get::<i64, _>("valid_until")
        .map_err(db::sql_error)?
        < now
    {
        return Err("approval-stale: approval has expired".into());
    }
    if row
        .try_get::<String, _>("action_id")
        .map_err(db::sql_error)?
        != action_id
        || row
            .try_get::<String, _>("plan_digest")
            .map_err(db::sql_error)?
            != plan_digest
    {
        return Err("approval-stale: action plan changed after approval".into());
    }
    let result = sqlx::query(
        "UPDATE spaces_action_approvals SET consumed_at=$1
         WHERE id=$2 AND consumed_at IS NULL",
    )
    .bind(now)
    .bind(approval_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(result, "approval-stale: approval was consumed concurrently")
}

pub(crate) async fn decide(
    pool: &sqlx::SqlitePool,
    approval_id: &str,
    decision: ApprovalDecision,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let value = match decision {
        ApprovalDecision::Approve => "approved",
        ApprovalDecision::Reject => "rejected",
    };
    let result = sqlx::query(
        "UPDATE spaces_action_approvals SET decision=$1,decided_at=$2
         WHERE id=$3 AND decision='pending' AND valid_until >= $2",
    )
    .bind(value)
    .bind(now)
    .bind(approval_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        result,
        "approval-stale: approval is unavailable or already decided",
    )
}

#[cfg(test)]
mod tests;

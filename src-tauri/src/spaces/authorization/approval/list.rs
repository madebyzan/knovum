use sqlx::Row;

use super::PendingApproval;
use crate::spaces::db;

pub(crate) async fn list_pending(
    pool: &sqlx::SqlitePool,
    space_id: &str,
) -> Result<Vec<PendingApproval>, String> {
    let rows = sqlx::query(
        "SELECT a.id,a.action_id,a.plan_digest,a.plan,a.expected_revisions,a.valid_until,a.created_at,
         COALESCE(
           (SELECT t.run_id FROM spaces_assistant_tasks t WHERE t.grant_id=a.grant_id),
           (SELECT e.run_id FROM spaces_external_agent_conversations e
            JOIN spaces_authorization_grants g ON g.principal_kind='external_agent'
              AND g.principal_id=e.id WHERE g.id=a.grant_id),
           (SELECT r.id FROM spaces_runs r
            WHERE json_extract(r.policy_snapshot,'$.grantId')=a.grant_id
            ORDER BY r.created_at DESC LIMIT 1)
         ) run_id
         FROM spaces_action_approvals a WHERE a.space_id=$1 AND a.decision='pending'
         AND a.valid_until >= $2 ORDER BY a.created_at",
    ).bind(space_id).bind(crate::clock::unix_time_millis()?)
        .fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            let encoded: String = row.try_get("expected_revisions").map_err(db::sql_error)?;
            let plan: String = row.try_get("plan").map_err(db::sql_error)?;
            Ok(PendingApproval {
                id: row.try_get("id").map_err(db::sql_error)?,
                run_id: row.try_get("run_id").map_err(db::sql_error)?,
                action_id: row.try_get("action_id").map_err(db::sql_error)?,
                plan_digest: row.try_get("plan_digest").map_err(db::sql_error)?,
                plan: serde_json::from_str(&plan).map_err(db::json_error)?,
                expected_revisions: serde_json::from_str(&encoded).map_err(db::json_error)?,
                valid_until: row.try_get("valid_until").map_err(db::sql_error)?,
                created_at: row.try_get("created_at").map_err(db::sql_error)?,
            })
        })
        .collect()
}

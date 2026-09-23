use serde_json::Value;
use sqlx::{Row, SqlitePool};

use crate::spaces::db;

pub(super) async fn rate_exceeded(
    pool: &SqlitePool,
    trigger_id: &str,
    limit: u16,
    window_seconds: u32,
    now: i64,
) -> Result<bool, String> {
    let cutoff = now.saturating_sub(i64::from(window_seconds) * 1_000);
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_automation_occurrences
         WHERE trigger_id=$1 AND created_at >= $2 AND occurrence_key LIKE 'event:%'",
    )
    .bind(trigger_id)
    .bind(cutoff)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(count >= i64::from(limit))
}

pub(super) async fn reserve(
    pool: &SqlitePool,
    event: &sqlx::sqlite::SqliteRow,
    trigger_id: &str,
    payload: &Value,
    trace: &[String],
    state: &str,
    now: i64,
) -> Result<(), String> {
    let event_id: String = event.try_get("id").map_err(db::sql_error)?;
    let mut next_trace = trace.to_vec();
    next_trace.push(trigger_id.to_owned());
    let detail = serde_json::json!({
        "eventId": event_id,
        "kind": event.try_get::<String, _>("kind").map_err(db::sql_error)?,
        "payload": payload,
        "causalRootId": event.try_get::<String, _>("causal_root_id").map_err(db::sql_error)?,
        "causalDepth": event.try_get::<i64, _>("causal_depth").map_err(db::sql_error)? + 1,
        "triggerTrace": next_trace,
    });
    sqlx::query(
        "INSERT OR IGNORE INTO spaces_automation_occurrences(
         id,trigger_id,scheduled_at,occurrence_key,run_id,state,detail,created_at)
         VALUES($1,$2,$3,$4,NULL,$5,$6,$3)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(trigger_id)
    .bind(now)
    .bind(format!("event:{event_id}"))
    .bind(state)
    .bind(serde_json::to_string(&detail).map_err(db::json_error)?)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

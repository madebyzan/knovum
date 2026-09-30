use serde_json::Value;
use sqlx::{Row, SqlitePool};

use crate::spaces::db;

pub(super) async fn resume(pool: &SqlitePool) -> Result<(), String> {
    let rows = sqlx::query("SELECT s.id,s.run_id,s.output,s.wait_reason FROM spaces_run_steps s JOIN spaces_runs r ON r.id=s.run_id WHERE s.state='waiting' AND r.state IN ('queued','running','waiting') AND r.cancel_requested_at IS NULL AND r.pause_requested_at IS NULL AND r.completed_at IS NULL")
        .fetch_all(pool).await.map_err(db::sql_error)?;
    for row in rows {
        let reason: Option<String> = row.try_get("wait_reason").map_err(db::sql_error)?;
        let output = parse_optional_json(&row)?;
        let settlement = match reason.as_deref() {
            Some("executor") => single_child_settlement(pool, output.as_ref()).await?,
            _ => None,
        };
        let Some(settlement) = settlement else {
            continue;
        };
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let run_id: String = row.try_get("run_id").map_err(db::sql_error)?;
        super::completion::settle_waiting_step(
            pool,
            &id,
            &run_id,
            settlement.as_ref().map_err(String::as_str),
        )
        .await?;
    }
    Ok(())
}

pub(super) async fn single_child_settlement(
    pool: &SqlitePool,
    output: Option<&Value>,
) -> Result<Option<Result<Value, String>>, String> {
    let Some(child_id) = output
        .and_then(|value| value.get("childRunId"))
        .and_then(Value::as_str)
    else {
        return Ok(None);
    };
    let row = sqlx::query("SELECT state,completed_at FROM spaces_runs WHERE id=$1")
        .bind(child_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?;
    let Some(row) = row else {
        return Ok(Some(Err("child-run-unavailable".into())));
    };
    if row
        .try_get::<Option<i64>, _>("completed_at")
        .map_err(db::sql_error)?
        .is_none()
    {
        return Ok(None);
    }
    let state: String = row.try_get("state").map_err(db::sql_error)?;
    if !matches!(state.as_str(), "succeeded" | "completed_with_errors") {
        return Ok(Some(Err(format!("child-run-{state}"))));
    }
    let value = sqlx::query_scalar::<_, Option<String>>("SELECT output FROM spaces_run_steps WHERE run_id=$1 AND state='succeeded' ORDER BY updated_at DESC LIMIT 1")
        .bind(child_id).fetch_optional(pool).await.map_err(db::sql_error)?.flatten()
        .map(|encoded| serde_json::from_str(&encoded).map_err(db::json_error)).transpose()?.unwrap_or(Value::Null);
    let child = serde_json::json!({"childRunId": child_id, "childState": state, "output": value});
    let mut resumed = output.cloned().unwrap_or(Value::Null);
    if let Some(object) = resumed.as_object_mut() {
        let output_node_id: Option<String> = sqlx::query_scalar(
            "SELECT output_node_id FROM spaces_knowledge_derivations WHERE id=$1 AND status='completed'",
        )
        .bind(child_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?;
        if let Some(node_id) = output_node_id {
            object.insert("nodeId".into(), node_id.into());
        }
        object.insert("childResult".into(), child);
        Ok(Some(Ok(resumed)))
    } else {
        Ok(Some(Ok(child)))
    }
}

fn parse_optional_json(row: &sqlx::sqlite::SqliteRow) -> Result<Option<Value>, String> {
    row.try_get::<Option<String>, _>("output")
        .map_err(db::sql_error)?
        .map(|encoded| serde_json::from_str(&encoded).map_err(db::json_error))
        .transpose()
}

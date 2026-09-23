use serde_json::Value;
use sqlx::{Row, SqlitePool};

use crate::spaces::db;

mod batch;

pub(super) async fn resume(pool: &SqlitePool) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let rows = sqlx::query("SELECT s.id,s.run_id,s.output,s.wait_reason FROM spaces_run_steps s JOIN spaces_runs r ON r.id=s.run_id WHERE s.state='waiting' AND r.completed_at IS NULL")
        .fetch_all(pool).await.map_err(db::sql_error)?;
    for row in rows {
        let reason: Option<String> = row.try_get("wait_reason").map_err(db::sql_error)?;
        let output = parse_optional_json(&row)?;
        let settlement = match reason.as_deref() {
            Some("executor") => child_settlement(pool, output.as_ref()).await?,
            Some("delay") if due(output.as_ref(), now) => {
                Some(Ok(serde_json::json!({"delayCompleted": true})))
            }
            Some("approval") => approval_settlement(pool, output.as_ref()).await?,
            _ => None,
        };
        let Some(settlement) = settlement else {
            continue;
        };
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let run_id: String = row.try_get("run_id").map_err(db::sql_error)?;
        if reason.as_deref() == Some("approval") && settlement.is_ok() {
            super::completion::resume_approved_step(pool, &id, &run_id).await?;
        } else if output
            .as_ref()
            .and_then(|value| value.get("resumeStep"))
            .and_then(Value::as_bool)
            == Some(true)
        {
            match settlement {
                Ok(value) => {
                    super::completion::resume_step_with_output(pool, &id, &run_id, &value).await?
                }
                Err(error) => {
                    super::completion::settle_waiting_step(pool, &id, &run_id, Err(&error)).await?
                }
            }
        } else {
            match settlement {
                Ok(value) => {
                    super::completion::settle_waiting_step(pool, &id, &run_id, Ok(&value)).await?
                }
                Err(error) => {
                    super::completion::settle_waiting_step(pool, &id, &run_id, Err(&error)).await?
                }
            }
        }
    }
    Ok(())
}

async fn approval_settlement(
    pool: &SqlitePool,
    output: Option<&Value>,
) -> Result<Option<Result<Value, String>>, String> {
    if let Some(approvals) = output
        .and_then(|value| value.get("approvals"))
        .and_then(Value::as_array)
    {
        for approval in approvals {
            let Some(id) = approval.get("approvalId").and_then(Value::as_str) else {
                return Ok(Some(Err("approval-checkpoint-invalid".into())));
            };
            match approval_decision(pool, id).await?.as_deref() {
                Some("approved") => {}
                Some("rejected") => return Ok(Some(Err("approval-rejected".into()))),
                Some("pending") => return Ok(None),
                _ => return Ok(Some(Err("approval-unavailable".into()))),
            }
        }
        return Ok(Some(Ok(output.cloned().unwrap_or(Value::Null))));
    }
    let Some(id) = output
        .and_then(|value| value.get("approvalId"))
        .and_then(Value::as_str)
    else {
        return Ok(Some(Err("approval-checkpoint-invalid".into())));
    };
    Ok(match approval_decision(pool, id).await?.as_deref() {
        Some("approved") => Some(Ok(output.cloned().unwrap_or(Value::Null))),
        Some("rejected") => Some(Err("approval-rejected".into())),
        Some("pending") => None,
        _ => Some(Err("approval-unavailable".into())),
    })
}

async fn approval_decision(pool: &SqlitePool, id: &str) -> Result<Option<String>, String> {
    sqlx::query_scalar("SELECT decision FROM spaces_action_approvals WHERE id=$1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)
}

async fn child_settlement(
    pool: &SqlitePool,
    output: Option<&Value>,
) -> Result<Option<Result<Value, String>>, String> {
    if let Some(children) = output
        .and_then(|value| value.get("childRuns"))
        .and_then(Value::as_array)
    {
        return batch::settlement(pool, output.unwrap_or(&Value::Null), children).await;
    }
    single_child_settlement(pool, output).await
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
fn due(output: Option<&Value>, now: i64) -> bool {
    output
        .and_then(|value| value.get("resumeAt"))
        .and_then(Value::as_i64)
        .is_some_and(|time| time <= now)
}

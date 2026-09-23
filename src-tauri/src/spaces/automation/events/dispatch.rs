use serde_json::Value;
use sqlx::{Row, SqlitePool};

use super::super::contract::AutomationSource;
use crate::spaces::{db, workflows};

mod reservation;

pub(super) async fn pending(pool: &SqlitePool, now: i64) -> Result<(), String> {
    let events = sqlx::query(
        "SELECT * FROM spaces_automation_events WHERE dispatched_at IS NULL
         ORDER BY occurred_at,id LIMIT 50",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for event in events {
        dispatch(pool, &event, now).await?;
        sqlx::query("UPDATE spaces_automation_events SET dispatched_at=$1 WHERE id=$2 AND dispatched_at IS NULL")
            .bind(now).bind(event.get::<String, _>("id"))
            .execute(pool).await.map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn reconcile(pool: &SqlitePool) -> Result<(), String> {
    let rows = sqlx::query(
        "SELECT a.id,a.result,r.home_space_id,r.input_snapshot,r.parent_run_id,a.finished_at
         FROM spaces_run_attempts a JOIN spaces_runs r ON r.id=a.run_id
         LEFT JOIN spaces_automation_events e ON e.id=a.id
         WHERE r.kind='action' AND a.state='succeeded' AND e.id IS NULL AND a.result IS NOT NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        let input = parse_json(&row, "input_snapshot")?;
        let action_id = input
            .get("actionId")
            .and_then(Value::as_str)
            .ok_or("action-run-invalid: actionId is missing")?;
        let mut tx = db::begin_write(pool).await?;
        super::record_action_event(
            &mut tx,
            &row.try_get::<String, _>("id").map_err(db::sql_error)?,
            &row.try_get::<String, _>("home_space_id")
                .map_err(db::sql_error)?,
            action_id,
            &parse_json(&row, "result")?,
            row.try_get::<Option<String>, _>("parent_run_id")
                .map_err(db::sql_error)?
                .as_deref(),
            row.try_get::<i64, _>("finished_at")
                .map_err(db::sql_error)?,
        )
        .await?;
        tx.commit().await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn dispatch(
    pool: &SqlitePool,
    event: &sqlx::sqlite::SqliteRow,
    now: i64,
) -> Result<(), String> {
    let space_id: String = event.try_get("space_id").map_err(db::sql_error)?;
    let kind: String = event.try_get("kind").map_err(db::sql_error)?;
    let triggers = sqlx::query(
        "SELECT id,source_config,valid_until FROM spaces_automation_triggers
         WHERE enabled=1 AND source_kind='event' AND space_id=$1",
    )
    .bind(&space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for trigger in triggers {
        dispatch_to_trigger(pool, event, &kind, &trigger, now).await?;
    }
    Ok(())
}

async fn dispatch_to_trigger(
    pool: &SqlitePool,
    event: &sqlx::sqlite::SqliteRow,
    event_kind: &str,
    trigger: &sqlx::sqlite::SqliteRow,
    now: i64,
) -> Result<(), String> {
    let trigger_id: String = trigger.try_get("id").map_err(db::sql_error)?;
    let source: String = trigger.try_get("source_config").map_err(db::sql_error)?;
    let AutomationSource::Event {
        event_kind: expected,
        filter_expression,
        causal_depth_limit,
        rate_limit_count,
        rate_limit_window_seconds,
    } = serde_json::from_str(&source).map_err(db::json_error)?
    else {
        return Ok(());
    };
    if expected.as_str() != event_kind {
        return Ok(());
    }
    let payload = parse_json(event, "payload")?;
    if !filter_matches(filter_expression.as_deref(), &payload)? {
        return Ok(());
    }
    let depth: u8 = event.try_get("causal_depth").map_err(db::sql_error)?;
    let trace: Vec<String> = serde_json::from_str(
        &event
            .try_get::<String, _>("trigger_trace")
            .map_err(db::sql_error)?,
    )
    .map_err(db::json_error)?;
    let state = if trace.contains(&trigger_id) || depth >= causal_depth_limit {
        "skipped_loop"
    } else if reservation::rate_exceeded(
        pool,
        &trigger_id,
        rate_limit_count,
        rate_limit_window_seconds,
        now,
    )
    .await?
    {
        "skipped_rate"
    } else if trigger
        .try_get::<i64, _>("valid_until")
        .map_err(db::sql_error)?
        < now
    {
        "authorization_expired"
    } else {
        "queued"
    };
    reservation::reserve(pool, event, &trigger_id, &payload, &trace, state, now).await
}

fn filter_matches(expression: Option<&str>, payload: &Value) -> Result<bool, String> {
    let Some(expression) = expression.filter(|value| !value.trim().is_empty()) else {
        return Ok(true);
    };
    workflows::expressions::evaluate(expression, payload)?
        .as_bool()
        .ok_or("automation-event-filter-invalid: filter must return boolean".into())
}

fn parse_json(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<Value, String> {
    serde_json::from_str(&row.try_get::<String, _>(field).map_err(db::sql_error)?)
        .map_err(db::json_error)
}

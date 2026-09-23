mod causality;
mod dispatch;

use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};

use super::contract::AutomationEventType;
use crate::spaces::db;

pub(crate) struct NewAutomationEvent<'a> {
    pub id: &'a str,
    pub space_id: &'a str,
    pub kind: &'a AutomationEventType,
    pub payload: &'a Value,
    pub causal_root_id: &'a str,
    pub causal_depth: u8,
    pub trigger_trace: &'a [String],
}

pub(crate) async fn record_action_event(
    tx: &mut Transaction<'_, Sqlite>,
    event_id: &str,
    space_id: &str,
    action_id: &str,
    payload: &Value,
    parent_run_id: Option<&str>,
    now: i64,
) -> Result<(), String> {
    let causality = causality::load(tx, parent_run_id).await?;
    let kind = AutomationEventType::action_committed(action_id)?;
    record_event(
        tx,
        &NewAutomationEvent {
            id: event_id,
            space_id,
            kind: &kind,
            payload,
            causal_root_id: &causality.root_id,
            causal_depth: causality.depth,
            trigger_trace: &causality.trace,
        },
        now,
    )
    .await
}

pub(crate) async fn record_event(
    tx: &mut Transaction<'_, Sqlite>,
    event: &NewAutomationEvent<'_>,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT OR IGNORE INTO spaces_automation_events(
         id,space_id,kind,payload,causal_root_id,causal_depth,trigger_trace,occurred_at)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(event.id)
    .bind(event.space_id)
    .bind(event.kind.as_str())
    .bind(serde_json::to_string(event.payload).map_err(db::json_error)?)
    .bind(event.causal_root_id)
    .bind(event.causal_depth)
    .bind(serde_json::to_string(event.trigger_trace).map_err(db::json_error)?)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(super) async fn dispatch_pending(pool: &SqlitePool, now: i64) -> Result<(), String> {
    dispatch::pending(pool, now).await
}

pub(super) async fn reconcile_action_events(pool: &SqlitePool) -> Result<(), String> {
    dispatch::reconcile(pool).await
}

use serde_json::json;

use super::identity;
use crate::spaces::{assistant::repository, db};

pub(super) async fn complete(
    pool: &sqlx::SqlitePool,
    ids: &identity::TaskIds,
    output: String,
) -> Result<(), String> {
    identity::ensure_active(pool, &ids.run_id).await?;
    let run = crate::spaces::runs::load_run(pool, &ids.run_id).await?;
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let updated = sqlx::query(
        "UPDATE spaces_runs SET state='succeeded',state_version=state_version+1,updated_at=$1,
         completed_at=$1,wait_reason=NULL WHERE id=$2 AND state_version=$3 AND state='running'
         AND cancel_requested_at IS NULL AND pause_requested_at IS NULL AND completed_at IS NULL",
    )
    .bind(now)
    .bind(&ids.run_id)
    .bind(run.state_version)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        updated,
        "assistant-task-inactive: task was cancelled or paused",
    )?;
    repository::append_message_in_transaction(
        &mut tx,
        &ids.conversation_id,
        &ids.branch_id,
        repository::AssistantMessageRole::Assistant,
        &json!({"parts": [{"type": "text", "text": output}], "runId": ids.run_id}),
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}

pub(super) async fn fail(pool: &sqlx::SqlitePool, run_id: &str, error: &str) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let control = sqlx::query_as::<_, (Option<i64>, Option<i64>)>(
        "SELECT pause_requested_at,cancel_requested_at FROM spaces_runs WHERE id=$1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    if let Some((pause_requested_at, cancel_requested_at)) = control {
        if pause_requested_at.is_some() && cancel_requested_at.is_none() {
            let mut tx = db::begin_write(pool).await?;
            crate::spaces::runs::append_event(
                &mut tx,
                run_id,
                None,
                None,
                crate::spaces::runs::RunEventKind::AssistantPausedCheckpoint,
                &json!({}),
                now,
            )
            .await?;
            tx.commit().await.map_err(db::sql_error)?;
            crate::spaces::runs::announce_change();
            return Ok(());
        }
        if cancel_requested_at.is_some() {
            return Ok(());
        }
    }
    let mut tx = db::begin_write(pool).await?;
    let task_write = sqlx::query(
        "UPDATE spaces_runs SET state='failed',state_version=state_version+1,updated_at=$1,completed_at=$1
         WHERE id=$2 AND completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL",
    ).bind(now).bind(run_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    if task_write.rows_affected() != 0 {
        let message = error.chars().take(4_000).collect::<String>();
        sqlx::query("UPDATE spaces_assistant_tasks SET error=$1,updated_at=$2 WHERE run_id=$3")
            .bind(message)
            .bind(now)
            .bind(run_id)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}

pub(super) async fn stopped_state(pool: &sqlx::SqlitePool, run_id: &str) -> String {
    let controls = sqlx::query_as::<_, (Option<i64>, Option<i64>)>(
        "SELECT pause_requested_at,cancel_requested_at FROM spaces_runs WHERE id=$1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match controls {
        Some((_, Some(_))) => "cancelled",
        Some((Some(_), None)) => "paused",
        _ => "failed",
    }
    .into()
}

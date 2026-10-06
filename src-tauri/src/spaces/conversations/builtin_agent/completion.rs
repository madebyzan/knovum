use serde_json::json;

use super::identity;
use crate::spaces::{conversations::repository, db};

pub(super) async fn complete(
    pool: &sqlx::SqlitePool,
    ids: &identity::TaskIds,
) -> Result<super::super::ConversationTaskFinishedState, String> {
    identity::ensure_active(pool, &ids.run_id).await?;
    let run = crate::spaces::runs::load_run(pool, &ids.run_id).await?;
    crate::spaces::conversations::execution::ledger::require_reconciled(pool, &ids.task_id).await?;
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let within_budget: bool = sqlx::query_scalar(
        "SELECT total_tokens<=json_extract(budget,'$.maxTotalTokens') FROM spaces_runs WHERE id=$1",
    )
    .bind(&ids.run_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if !within_budget {
        return Err("budget-exhausted: Builtin Agent model token budget was exceeded".into());
    }
    let children = crate::spaces::runs::child_settlement(&mut tx, &ids.run_id)
        .await?
        .ok_or("builtin-agent-child-runs-unsettled")?;
    let (run_state, finished_state) = if children == crate::spaces::runs::RunState::Succeeded {
        (
            "succeeded",
            super::super::ConversationTaskFinishedState::Succeeded,
        )
    } else {
        (
            "completed_with_errors",
            super::super::ConversationTaskFinishedState::CompletedWithErrors,
        )
    };
    let updated = sqlx::query(
        "UPDATE spaces_runs SET state=$4,state_version=state_version+1,updated_at=$1,
         completed_at=$1,wait_reason=NULL WHERE id=$2 AND state_version=$3 AND state='running'
         AND cancel_requested_at IS NULL AND pause_requested_at IS NULL AND completed_at IS NULL",
    )
    .bind(now)
    .bind(&ids.run_id)
    .bind(run.state_version)
    .bind(run_state)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        updated,
        "builtin-agent-task-inactive: task was cancelled or paused",
    )?;
    let principal = crate::spaces::authorization::Principal {
        kind: crate::spaces::authorization::PrincipalKind::BuiltinAgent,
        id: ids.task_id.clone(),
    };
    let outputs =
        crate::spaces::conversations::outputs::collect(&mut tx, &run.home_space_id, &principal)
            .await?;
    let last: Option<(String, String)> = sqlx::query_as("SELECT id,content FROM spaces_conversation_messages WHERE conversation_id=$1 AND branch_id=$2 AND role='assistant'
        AND json_extract(content,'$.runId')=$3 ORDER BY created_at DESC,rowid DESC LIMIT 1")
        .bind(&ids.conversation_id).bind(&ids.branch_id).bind(&ids.run_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
    if let Some((id, content)) = last {
        let mut content: serde_json::Value =
            serde_json::from_str(&content).map_err(db::json_error)?;
        content["outputs"] = serde_json::to_value(outputs).map_err(db::json_error)?;
        sqlx::query("UPDATE spaces_conversation_messages SET content=$1 WHERE id=$2")
            .bind(serde_json::to_string(&content).map_err(db::json_error)?)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    } else if !outputs.is_empty() {
        repository::append_message_in_transaction(
            &mut tx,
            &ids.conversation_id,
            &ids.branch_id,
            repository::ConversationMessageRole::Assistant,
            &json!({"parts":[],"runId":ids.run_id,"outputs":outputs}),
            now,
        )
        .await?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(finished_state)
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
    if control.is_some_and(|(_, cancel)| cancel.is_some()) {
        return Ok(());
    }
    if let Some(reason) = super::model_recovery::reason(pool, run_id).await? {
        return super::model_recovery::wait(pool, run_id, reason).await;
    }
    if control.is_some_and(|(pause, _)| pause.is_some()) {
        let unaccepted: Option<String> = sqlx::query_scalar(
            "SELECT segment_id FROM spaces_builtin_agent_tasks WHERE run_id=$1 AND input_accepted=0",
        )
        .bind(run_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?;
        if let Some(thread_id) = unaccepted {
            let mut tx = db::begin_write(pool).await?;
            crate::spaces::runs::append_event(
                &mut tx,
                run_id,
                None,
                None,
                crate::spaces::runs::RunEventKind::BuiltinAgentPausedCheckpoint,
                &json!({"threadId":thread_id,"boundary":"host_input"}),
                now,
            )
            .await?;
            sqlx::query("UPDATE spaces_runs SET state='paused',wait_reason=NULL,state_version=state_version+1,updated_at=$1
                WHERE id=$2 AND completed_at IS NULL AND cancel_requested_at IS NULL")
                .bind(now).bind(run_id).execute(&mut *tx).await.map_err(db::sql_error)?;
            tx.commit().await.map_err(db::sql_error)?;
            crate::spaces::runs::announce_change();
            return Ok(());
        }
        // An accepted input without a settled graph boundary cannot claim pause.
        sqlx::query(
            "UPDATE spaces_runs SET pause_requested_at=NULL WHERE id=$1 AND completed_at IS NULL",
        )
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    }
    let mut tx = db::begin_write(pool).await?;
    let task_write = sqlx::query(
        "UPDATE spaces_runs SET state='failed',state_version=state_version+1,updated_at=$1,completed_at=$1
         WHERE id=$2 AND completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL",
    ).bind(now).bind(run_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    if task_write.rows_affected() != 0 {
        let message = error.chars().take(4_000).collect::<String>();
        sqlx::query("UPDATE spaces_builtin_agent_tasks SET error=$1,updated_at=$2 WHERE run_id=$3")
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

pub(super) async fn stopped_state(
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<super::super::contract::ConversationTaskFinishedState, String> {
    use super::super::contract::ConversationTaskFinishedState;
    let controls = sqlx::query_as::<_, (Option<i64>, Option<i64>)>(
        "SELECT pause_requested_at,cancel_requested_at FROM spaces_runs WHERE id=$1",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    if crate::spaces::runs::load_run(pool, run_id).await?.state
        == crate::spaces::runs::RunState::Waiting
    {
        return Ok(ConversationTaskFinishedState::Waiting);
    }
    Ok(match controls {
        (_, Some(_)) => ConversationTaskFinishedState::Cancelled,
        (Some(_), None) => ConversationTaskFinishedState::Paused,
        _ => ConversationTaskFinishedState::Failed,
    })
}

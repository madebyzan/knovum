use serde_json::Value;
use sqlx::{Row, SqlitePool};

use super::LocalActionOperationOutcome;
use crate::spaces::db;

pub struct LocalActionOperationStart<'a> {
    pub task_id: Option<&'a str>,
    pub run_id: &'a str,
    pub graph_namespace: Option<&'a str>,
    pub checkpoint_id: Option<&'a str>,
    pub message_id: Option<&'a str>,
    pub provider_call_id: &'a str,
    pub wire_name: &'a str,
    pub action_id: &'a str,
    pub invocation_id: &'a str,
    pub arguments: &'a Value,
    pub target_selection: Option<Value>,
}

pub(in crate::spaces) async fn start_tool_call(
    pool: &SqlitePool,
    call: LocalActionOperationStart<'_>,
) -> Result<LocalActionOperationOutcome, String> {
    let LocalActionOperationStart {
        task_id,
        run_id,
        graph_namespace,
        checkpoint_id,
        message_id,
        provider_call_id,
        wire_name,
        action_id,
        invocation_id,
        arguments,
        target_selection,
    } = call;
    let id = uuid::Uuid::new_v4().to_string();
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let previous: Option<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_conversation_tool_operations WHERE invocation_id=$1",
    )
    .bind(invocation_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if previous.is_none() {
        let allowed: Option<i64> = sqlx::query_scalar("SELECT 1 FROM spaces_runs r WHERE id=$1 AND completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL AND $2 < created_at+json_extract(budget,'$.maxDurationSeconds')*1000 AND (SELECT count(*) FROM spaces_conversation_tool_operations WHERE run_id=r.id) < json_extract(budget,'$.maxTools')")
            .bind(run_id).bind(now).fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
        allowed.ok_or("conversation-tool-budget-exhausted")?;
        if crate::spaces::actions::require(action_id)?.impact
            != crate::spaces::actions::SpaceActionImpact::ReadOnly
        {
            super::require_reconciled_on(&mut tx, run_id).await?;
        }
    }
    let inserted = sqlx::query("INSERT OR IGNORE INTO spaces_conversation_tool_operations(id,task_id,graph_namespace,message_id,provider_call_id,
         wire_name,source,invocation_id,target_selection,created_at,run_id,checkpoint_id,context,context_digest,read_only,approval) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,'allow')")
        .bind(&id).bind(task_id).bind(graph_namespace).bind(message_id).bind(provider_call_id).bind(wire_name)
        .bind(sqlx::types::Json(crate::extensions::contracts::AgentToolBinding::SpaceAction { action_id: action_id.into(), action_version: crate::spaces::actions::require(action_id)?.version })).bind(invocation_id).bind(target_selection.as_ref().map(sqlx::types::Json)).bind(now).bind(run_id).bind(checkpoint_id).bind(sqlx::types::Json(arguments)).bind(super::context_digest(arguments)?).bind(crate::spaces::actions::require(action_id)?.impact == crate::spaces::actions::SpaceActionImpact::ReadOnly)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    if inserted.rows_affected() != 0 {
        crate::spaces::runs::append_event(
            &mut tx,
            run_id,
            None,
            None,
            crate::spaces::runs::RunEventKind::ConversationToolStarted,
            &serde_json::json!({
                "toolCallId": provider_call_id,
                "actionId": action_id,
                "context": arguments,
            }),
            now,
        )
        .await?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    if inserted.rows_affected() != 0 {
        crate::spaces::runs::announce_change();
    }
    let row = sqlx::query("SELECT id,output,target_selection,context_digest,run_id,source FROM spaces_conversation_tool_operations WHERE invocation_id=$1 ORDER BY attempt DESC LIMIT 1")
        .bind(invocation_id).fetch_one(pool).await.map_err(db::sql_error)?;
    let binding: crate::extensions::contracts::AgentToolBinding =
        serde_json::from_str(&row.try_get::<String, _>("source").map_err(db::sql_error)?)
            .map_err(db::json_error)?;
    if row
        .try_get::<String, _>("context_digest")
        .map_err(db::sql_error)?
        != super::context_digest(arguments)?
        || row.try_get::<String, _>("run_id").map_err(db::sql_error)? != run_id
        || !matches!(binding, crate::extensions::contracts::AgentToolBinding::SpaceAction { action_id: ref stored, .. } if stored == action_id)
    {
        return Err("conversation-tool-intent-identity-changed".into());
    }
    let stored_id: String = row.try_get("id").map_err(db::sql_error)?;
    let tool_output = row
        .try_get::<Option<String>, _>("output")
        .map_err(db::sql_error)?
        .map(|value| serde_json::from_str(&value).map_err(db::json_error))
        .transpose()?;
    Ok(LocalActionOperationOutcome {
        id: stored_id,
        output: tool_output,
        target_selection: row
            .try_get::<Option<String>, _>("target_selection")
            .map_err(db::sql_error)?
            .map(|value| serde_json::from_str(&value).map_err(db::json_error))
            .transpose()?,
    })
}

pub async fn complete_tool_call(
    pool: &SqlitePool,
    operation_id: &str,
    tool_output: &Value,
) -> Result<(), String> {
    super::receipts::complete(
        pool,
        operation_id,
        if tool_output["isError"] == true {
            "failed"
        } else {
            "completed"
        },
        tool_output,
    )
    .await
}

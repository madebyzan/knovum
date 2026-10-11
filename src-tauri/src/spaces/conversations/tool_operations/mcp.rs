use crate::{
    extensions::contracts::{AgentExtensionSnapshot, AgentExtensionTool, McpToolPolicy},
    spaces::{db, runs},
};
use serde_json::Value;
use sqlx::{Row, SqlitePool};
use std::time::Duration;
use tauri::AppHandle;

pub(crate) struct ToolInvocation<'a> {
    pub run_id: &'a str,
    pub intent_id: &'a str,
    pub call_id: &'a str,
    pub task_id: Option<&'a str>,
    pub graph_namespace: Option<&'a str>,
    pub checkpoint_id: Option<&'a str>,
    pub message_id: Option<&'a str>,
    pub tool: &'a AgentExtensionTool,
    pub arguments: Value,
}

pub(crate) enum ToolInvocationOutcome {
    Completed(Value),
    AwaitUser {
        operation_id: String,
        reason: &'static str,
    },
}

pub(crate) async fn invoke(
    app: &AppHandle,
    pool: &SqlitePool,
    snapshot: &AgentExtensionSnapshot,
    call: ToolInvocation<'_>,
) -> Result<ToolInvocationOutcome, String> {
    let _intent = super::concurrency::acquire(app, call.intent_id).await;
    crate::extensions::snapshots::require_current(app, snapshot).await?;
    if call.tool.policy == McpToolPolicy::Deny {
        return Err("mcp-tool-denied".into());
    }
    let run = runs::load_run(pool, call.run_id).await?;
    if run.state.is_terminal() || run.state == runs::RunState::Paused {
        return Err("conversation-tool-run-inactive".into());
    }
    let digest = super::context_digest(&call.arguments)?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let existing: Option<String> = sqlx::query_scalar("SELECT id FROM spaces_conversation_tool_operations WHERE invocation_id=$1 ORDER BY attempt DESC LIMIT 1")
        .bind(call.intent_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
    if existing.is_none() {
        let allowed: Option<i64> = sqlx::query_scalar("SELECT 1 FROM spaces_runs r WHERE id=$1 AND completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL
            AND $2 < created_at+json_extract(budget,'$.maxDurationSeconds')*1000
            AND (SELECT count(*) FROM spaces_conversation_tool_operations WHERE run_id=r.id) < json_extract(budget,'$.maxTools')")
            .bind(call.run_id).bind(now).fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
        allowed.ok_or("conversation-tool-budget-exhausted")?;
        if !call.tool.read_only {
            super::require_reconciled_on(&mut tx, call.run_id).await?;
        }
        sqlx::query("INSERT INTO spaces_conversation_tool_operations(id,run_id,task_id,graph_namespace,checkpoint_id,message_id,provider_call_id,wire_name,source,invocation_id,context,context_digest,read_only,approval,created_at)
            VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)")
            .bind(&id).bind(call.run_id).bind(call.task_id).bind(call.graph_namespace).bind(call.checkpoint_id).bind(call.message_id).bind(call.call_id).bind(&call.tool.name)
            .bind(sqlx::types::Json(&call.tool.binding)).bind(call.intent_id).bind(sqlx::types::Json(&call.arguments)).bind(&digest)
            .bind(call.tool.read_only).bind((call.tool.policy == McpToolPolicy::Allow).then_some("allow")).bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
        runs::append_event(&mut tx,call.run_id,None,None,runs::RunEventKind::ConversationToolStarted,&serde_json::json!({"toolCallId":call.call_id,"title":call.tool.name,"context":{"source":call.tool.binding,"arguments":call.arguments}}),now).await?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    let id = existing.unwrap_or(id);
    let row = sqlx::query("SELECT state,output,approval,context_digest,source,run_id,wire_name FROM spaces_conversation_tool_operations WHERE id=$1").bind(&id).fetch_one(pool).await.map_err(db::sql_error)?;
    if row
        .try_get::<String, _>("context_digest")
        .map_err(db::sql_error)?
        != digest
    {
        return Err("conversation-tool-intent-context-changed".into());
    }
    if row.try_get::<String, _>("run_id").map_err(db::sql_error)? != call.run_id
        || row
            .try_get::<String, _>("wire_name")
            .map_err(db::sql_error)?
            != call.tool.name
        || row
            .try_get::<sqlx::types::Json<serde_json::Value>, _>("source")
            .map_err(db::sql_error)?
            .0
            != serde_json::to_value(&call.tool.binding).map_err(db::json_error)?
    {
        return Err("conversation-tool-intent-source-changed".into());
    }
    let state: String = row.try_get("state").map_err(db::sql_error)?;
    if matches!(state.as_str(), "completed" | "failed") {
        return Ok(ToolInvocationOutcome::Completed(
            row.try_get::<sqlx::types::Json<serde_json::Value>, _>("output")
                .map_err(db::sql_error)?
                .0,
        ));
    }
    if matches!(state.as_str(), "dispatching" | "outcome_unknown") {
        return Ok(ToolInvocationOutcome::AwaitUser {
            operation_id: id,
            reason: "tool_outcome_unknown",
        });
    }
    if row
        .try_get::<Option<String>, _>("approval")
        .map_err(db::sql_error)?
        .is_none()
    {
        return Ok(ToolInvocationOutcome::AwaitUser {
            operation_id: id,
            reason: "tool_approval",
        });
    }
    let interaction = std::sync::Arc::new(super::interactions::RunMcpInteraction {
        pool: pool.clone(),
        run_id: call.run_id.into(),
    });
    let prepared = match crate::extensions::mcp::invocation::prepare(
        app,
        snapshot,
        call.tool,
        call.arguments,
        Some(interaction),
    )
    .await
    {
        Ok(prepared) => prepared,
        Err(error) => {
            let value = super::receipts::failure(error);
            super::receipts::complete(pool, &id, "failed", &value).await?;
            return Ok(ToolInvocationOutcome::Completed(value));
        }
    };
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    if !call.tool.read_only {
        super::require_reconciled_on(&mut tx, call.run_id).await?;
    }
    let remaining: i64 = sqlx::query_scalar("SELECT created_at+json_extract(budget,'$.maxDurationSeconds')*1000-$2 FROM spaces_runs WHERE id=$1")
        .bind(call.run_id).bind(now).fetch_one(&mut *tx).await.map_err(db::sql_error)?;
    if remaining <= 0 {
        return Err("conversation-tool-duration-exhausted".into());
    }
    let dispatched = sqlx::query("UPDATE spaces_conversation_tool_operations SET state='dispatching' WHERE id=$1 AND state='prepared' AND approval='allow'
        AND EXISTS(SELECT 1 FROM spaces_runs r WHERE r.id=run_id AND r.completed_at IS NULL AND r.cancel_requested_at IS NULL AND r.pause_requested_at IS NULL)")
        .bind(&id).execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(dispatched, "conversation-tool-dispatch-inactive")?;
    tx.commit().await.map_err(db::sql_error)?;
    let execution = prepared
        .execute(&id, Duration::from_millis(remaining.min(120_000) as u64))
        .await;
    let (state, value) = match execution {
        crate::extensions::mcp::invocation::InvocationObservation::Received(value) => (
            if value["isError"] == true {
                "failed"
            } else {
                "completed"
            },
            value,
        ),
        crate::extensions::mcp::invocation::InvocationObservation::OutcomeUnknown(error) => {
            ("outcome_unknown", super::receipts::failure(error))
        }
    };
    super::receipts::complete(pool, &id, state, &value).await?;
    if state == "outcome_unknown" {
        return Ok(ToolInvocationOutcome::AwaitUser {
            operation_id: id,
            reason: "tool_outcome_unknown",
        });
    }
    Ok(ToolInvocationOutcome::Completed(value))
}

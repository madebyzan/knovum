use serde_json::json;
use sqlx::SqlitePool;

#[cfg(test)]
mod tests;

use super::{identity, NewBuiltinAgentTask, BUILTIN_AGENT_PROFILE_VERSION};
use crate::spaces::{
    authorization::{create_grant, AuthorizationContext},
    conversations::{repository, tools, StartBuiltinAgentTaskRequest},
    db,
    runs::{create_run_in_transaction, NewRun, RunKind},
};

pub(super) struct NewBuiltinAgentTaskState<'a> {
    pub request: &'a StartBuiltinAgentTaskRequest,
    pub ids: &'a identity::TaskIds,
    pub grant: &'a AuthorizationContext,
    pub catalog: &'a tools::ToolCatalog,
    pub extensions: &'a crate::extensions::contracts::AgentExtensionSnapshot,
    pub context_manifest: &'a serde_json::Value,
    pub space_context: &'a crate::spaces::queries::knowledge_projection::ContentsPage,
    pub attachment_summaries: &'a [crate::spaces::conversations::ConversationMessageAttachment],
    pub target: &'a crate::ai::models::ModelExecutionTarget,
    pub profile: &'a crate::ai::settings::contracts::BuiltinAgentModelSettings,
    pub messages: &'a [crate::agent_runtime::protocol::messages::AgentInputMessage],
    pub history: &'a [crate::agent_runtime::protocol::messages::AgentInputMessage],
    pub now: i64,
}

pub(super) async fn persist_task(
    pool: &SqlitePool,
    records: NewBuiltinAgentTaskState<'_>,
) -> Result<
    (
        crate::spaces::conversations::execution::session::Session,
        Vec<crate::agent_runtime::protocol::messages::AgentInputMessage>,
    ),
    String,
> {
    let NewBuiltinAgentTaskState {
        request,
        ids,
        grant,
        catalog,
        extensions,
        context_manifest,
        space_context,
        attachment_summaries,
        target,
        profile,
        messages,
        history,
        now,
    } = records;
    let mut tx = db::begin_write(pool).await?;
    repository::ensure_branch_idle(&mut tx, &ids.conversation_id, &ids.branch_id).await?;
    repository::ensure_conversation(
        &mut tx,
        &ids.conversation_id,
        &request.space_id,
        &identity::conversation_title(&request.prompt),
        now,
    )
    .await?;
    sqlx::query("UPDATE spaces_conversations SET active_branch_id=$1 WHERE id=$2")
        .bind(&ids.branch_id)
        .bind(&ids.conversation_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    create_grant(&mut tx, grant, now).await?;
    let policy = json!({
        "grantId": grant.grant_id,
        "builtinAgentProfileVersion": BUILTIN_AGENT_PROFILE_VERSION,
    });
    let session = crate::spaces::conversations::execution::session::select(
        &mut tx,
        ids,
        target,
        profile,
        &request.target,
        &catalog.digest,
        now,
    )
    .await?;
    let mut inputs = if session.projection_checkpoint_id.is_none() {
        history.to_vec()
    } else {
        Vec::new()
    };
    inputs.extend_from_slice(messages);
    let input = json!({"prompt": request.prompt, "extensions": extensions, "contextManifest": context_manifest, "spaceContext": space_context, "messages": inputs});
    let budget = json!({
        "maxTurns": request.max_turns, "maxTools": request.max_tool_calls,
        "maxTotalTokens": request.max_total_tokens,
        "maxDurationSeconds": request.max_duration_seconds,
    });
    create_run_in_transaction(
        &mut tx,
        &NewRun {
            id: &ids.run_id,
            scope: &grant.scope,
            parent_run_id: None,
            principal: &grant.principal,
            kind: RunKind::BuiltinAgent,
            policy_snapshot: &policy,
            input_snapshot: &input,
            budget: &budget,
        },
        now,
    )
    .await?;
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_builtin_agent_tasks t JOIN spaces_runs r ON r.id=t.run_id WHERE r.completed_at IS NULL AND r.state IN ('queued','running','waiting')")
        .fetch_one(&mut *tx).await.map_err(db::sql_error)?;
    if active != 0 {
        return Err("builtin-agent-busy: another built-in execution is active".into());
    }
    repository::insert_task(
        &mut tx,
        &NewBuiltinAgentTask {
            task_id: &ids.task_id,
            run_id: &ids.run_id,
            conversation_id: &ids.conversation_id,
            branch_id: &ids.branch_id,
            grant_id: &ids.grant_id,
            model_binding: &request.target,
            catalog_digest: &catalog.digest,
            session: &session,
        },
        now,
    )
    .await?;
    repository::append_message_in_transaction(
        &mut tx,
        &ids.conversation_id,
        &ids.branch_id,
        repository::ConversationMessageRole::User,
        &json!({
            "parts": [{"type": "text", "text": request.prompt}],
            "runId": ids.run_id,
            "attachments": attachment_summaries,
        }),
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok((session, inputs))
}

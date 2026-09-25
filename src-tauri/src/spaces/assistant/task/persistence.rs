use serde_json::json;
use sqlx::SqlitePool;

use super::{identity, NewAssistantTask, ASSISTANT_PROFILE_VERSION};
use crate::{
    agent_runtime::ControlCheckpoint,
    spaces::{
        assistant::{repository, tools, StartAssistantTaskRequest},
        authorization::{create_grant, AuthorizationContext},
        db,
        runs::{create_run_in_transaction, NewRun, RunKind},
    },
};

pub(super) struct NewAssistantRecords<'a> {
    pub request: &'a StartAssistantTaskRequest,
    pub ids: &'a identity::TaskIds,
    pub grant: &'a AuthorizationContext,
    pub catalog: &'a tools::ToolCatalog,
    pub context_manifest: &'a serde_json::Value,
    pub checkpoint: &'a ControlCheckpoint,
    pub now: i64,
}

pub(super) async fn create_records(
    pool: &SqlitePool,
    records: NewAssistantRecords<'_>,
) -> Result<(), String> {
    let NewAssistantRecords {
        request,
        ids,
        grant,
        catalog,
        context_manifest,
        checkpoint,
        now,
    } = records;
    let mut tx = db::begin_write(pool).await?;
    repository::ensure_branch_idle(&mut tx, &ids.conversation_id, &ids.branch_id).await?;
    repository::ensure_conversation(
        &mut tx,
        &ids.conversation_id,
        &request.space_id,
        &identity::conversation_title(&request.prompt),
        request.mode,
        now,
    )
    .await?;
    sqlx::query("UPDATE spaces_assistant_conversations SET active_branch_id=$1 WHERE id=$2")
        .bind(&ids.branch_id)
        .bind(&ids.conversation_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    create_grant(&mut tx, grant, now).await?;
    let policy = json!({
        "mode": request.mode, "grantId": grant.grant_id,
        "assistantProfileVersion": ASSISTANT_PROFILE_VERSION,
    });
    let input = json!({"prompt": request.prompt, "contextManifest": context_manifest});
    let budget = json!({
        "maxTurns": request.max_turns, "maxTools": request.max_tool_calls,
        "maxTotalTokens": request.max_total_tokens,
        "maxDurationSeconds": request.max_duration_seconds,
    });
    create_run_in_transaction(
        &mut tx,
        &NewRun {
            id: &ids.run_id,
            home_space_id: &request.space_id,
            parent_run_id: None,
            principal: &grant.principal,
            kind: RunKind::Assistant,
            policy_snapshot: &policy,
            input_snapshot: &input,
            budget: &budget,
        },
        now,
    )
    .await?;
    let binding = serde_json::to_string(&request.target).map_err(db::json_error)?;
    repository::insert_task(
        &mut tx,
        &NewAssistantTask {
            task_id: &ids.task_id,
            run_id: &ids.run_id,
            conversation_id: &ids.conversation_id,
            branch_id: &ids.branch_id,
            grant_id: &ids.grant_id,
            model_binding: &binding,
            catalog_digest: &catalog.digest,
        },
        now,
    )
    .await?;
    repository::append_message_in_transaction(
        &mut tx,
        &ids.conversation_id,
        &ids.branch_id,
        "user",
        &json!({"parts": [{"type": "text", "text": request.prompt}]}),
        now,
    )
    .await?;
    sqlx::query(
        "INSERT INTO spaces_assistant_checkpoints(task_id,envelope,updated_at) VALUES($1,$2,$3)",
    )
    .bind(&ids.task_id)
    .bind(serde_json::to_string(checkpoint).map_err(db::json_error)?)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}

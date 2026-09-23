use serde_json::json;
use sqlx::SqlitePool;

use super::{identity, NewAssistantTask, ASSISTANT_PROFILE_VERSION};
use crate::spaces::{
    assistant::{repository, tools, StartAssistantTaskRequest},
    authorization::{create_grant, AuthorizationContext},
    db,
    runs::{create_run_in_transaction, NewRun, RunKind},
};

pub(super) async fn create_records(
    pool: &SqlitePool,
    request: &StartAssistantTaskRequest,
    ids: &identity::TaskIds,
    grant: &AuthorizationContext,
    catalog: &tools::ToolCatalog,
    context_manifest: &serde_json::Value,
    now: i64,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_assistant_tasks t JOIN spaces_runs r ON r.id=t.run_id \
         WHERE t.conversation_id=$1 AND t.branch_id=$2 AND r.completed_at IS NULL",
    )
    .bind(&ids.conversation_id)
    .bind(&ids.branch_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if active != 0 {
        return Err(
            "assistant-branch-busy: finish or cancel the active task before sending a follow-up"
                .into(),
        );
    }
    repository::ensure_conversation(
        &mut tx,
        &ids.conversation_id,
        &request.space_id,
        &identity::conversation_title(&request.prompt),
        request.mode,
        now,
    )
    .await?;
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
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}

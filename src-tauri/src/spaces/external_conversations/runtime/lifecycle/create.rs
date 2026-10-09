use tauri::AppHandle;

use super::super::StartExternalConversationRequest;
use crate::spaces::{
    authorization::{Principal, PrincipalKind, RunScope},
    db,
    runs::{NewRun, RunKind},
};

pub(crate) async fn create_records(
    app: &AppHandle,
    request: &StartExternalConversationRequest,
    binding_id: &str,
    run_id: &str,
    conversation: Option<(&str, &str)>,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let principal = Principal {
        kind: PrincipalKind::ExternalAgent,
        id: binding_id.to_owned(),
    };
    let scope = RunScope::Space {
        space_id: request.space_id.clone(),
    };
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let (conversation_id, branch_id) = conversation.unwrap_or((binding_id, binding_id));
    if conversation.is_some() {
        let owns_conversation: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM spaces_conversations WHERE id=$1 AND space_id=$2")
                .bind(conversation_id)
                .bind(&request.space_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(db::sql_error)?;
        if owns_conversation.is_none() {
            return Err("conversation-unavailable: conversation belongs to another Space".into());
        }
        let latest_branch: Option<String> =
            sqlx::query_scalar("SELECT active_branch_id FROM spaces_conversations WHERE id=$1")
                .bind(conversation_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(db::sql_error)?;
        if latest_branch.as_deref() != Some(branch_id) {
            return Err("conversation-branch-stale: select the current conversation branch".into());
        }
        crate::spaces::conversations::ensure_branch_idle(&mut tx, conversation_id, branch_id)
            .await?;
    }
    crate::spaces::runs::create_run_in_transaction(
        &mut tx,
        &NewRun {
            id: run_id,
            scope: &scope,
            parent_run_id: None,
            principal: &principal,
            kind: RunKind::ExternalAgent,
            policy_snapshot: &serde_json::json!({"runtimeKind": request.runtime_kind}),
            input_snapshot: &request.input_snapshot,
            budget: &serde_json::json!({"maxDurationSeconds": request.max_duration_seconds, "maxTools": 32}),
        },
        now,
    )
    .await?;
    if conversation.is_none() {
        sqlx::query(
            "INSERT INTO spaces_conversations(id,space_id,title,active_branch_id,created_at,updated_at)
             VALUES($1,$2,$3,$1,$4,$4)",
        )
        .bind(conversation_id)
        .bind(&request.space_id)
        .bind(request.title.trim())
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    }
    sqlx::query(
        "INSERT INTO spaces_external_agent_conversations(
         id,run_id,space_id,conversation_id,branch_id,runtime_kind,title,state,created_at,updated_at)
         VALUES($1,$2,$3,$4,$5,$6,$7,'connecting',$8,$8)",
    )
    .bind(binding_id)
    .bind(run_id)
    .bind(&request.space_id)
    .bind(conversation_id)
    .bind(branch_id)
    .bind(request.runtime_kind.as_str())
    .bind(request.title.trim())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}

mod contract;
mod lifecycle;
mod publication;
mod publication_ownership;
mod repository;
mod runtime;
mod staging;
mod updates;

pub(crate) use contract::{ExternalAgentSessionBinding, StartExternalConversationRequest};

pub(crate) fn recover(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = lifecycle::recover(app).await {
            log::error!("External Agent conversation recovery failed: {error}");
        }
    });
}

pub(crate) async fn cancel_by_run(app: &tauri::AppHandle, run_id: &str) -> Result<bool, String> {
    lifecycle::cancel_by_run(app, run_id).await
}

pub(crate) async fn purge_for_conversation(
    app: &tauri::AppHandle,
    conversation_id: &str,
) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT id,state FROM spaces_external_agent_conversations
         WHERE conversation_id=$1",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    if rows
        .iter()
        .any(|(_, state)| state == "connected" || state == "connecting")
    {
        return Err("conversation-busy".into());
    }
    for (id, _) in rows {
        lifecycle::delete(app, &id).await?;
    }
    Ok(())
}

pub(crate) async fn start_for_conversation(
    app: tauri::AppHandle,
    request: StartExternalConversationRequest,
    conversation: Option<(String, String)>,
    updates: tauri::ipc::Channel<crate::spaces::conversations::ConversationTaskUpdate>,
) -> Result<ExternalAgentSessionBinding, String> {
    runtime::start(app, request, conversation, Some(updates)).await
}

pub(crate) async fn begin_turn_for_conversation(
    app: tauri::AppHandle,
    session_id: &str,
    prompt: &str,
    attachments: &[crate::spaces::conversations::ConversationMessageAttachment],
) -> Result<String, String> {
    runtime::begin_turn(app, session_id, prompt, attachments).await
}

pub(crate) async fn send_for_conversation(
    app: tauri::AppHandle,
    session_id: &str,
    turn_id: &str,
    prepared: crate::external_agents::prompt::AgentPrompt,
) -> Result<(), String> {
    runtime::send_prepared(app, session_id, turn_id, prepared).await
}

pub(crate) async fn close_for_conversation(
    app: tauri::AppHandle,
    session_id: &str,
) -> Result<(), String> {
    lifecycle::close(app, session_id)
        .await
        .map_err(|error| error.to_string())
}

pub(crate) async fn fail_for_conversation(
    app: &tauri::AppHandle,
    session_id: &str,
    run_id: &str,
    error: &str,
) -> Result<(), String> {
    let mut close_error = None;
    if crate::external_agents::conversation::is_active(session_id) {
        close_error = crate::external_agents::conversation::close(session_id)
            .await
            .err()
            .map(|error| error.to_string());
    }
    let settlement = match crate::spaces::db::pool(app).await {
        Ok(pool) => runtime::lifecycle::mark_disconnected(pool, session_id, run_id, error).await,
        Err(error) => Err(error),
    };
    if let Some(close_error) = close_error {
        log::warn!(
            "External Agent runtime cleanup failed for session {session_id}, run {run_id}: {close_error}"
        );
    }
    settlement.map(|_| ())
}

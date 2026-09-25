mod contract;
mod lifecycle;
mod repository;
mod runtime;
mod updates;

pub(crate) use contract::{AgentSessionBinding, StartExternalConversationRequest};

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

pub(crate) async fn purge_for_assistant(
    app: &tauri::AppHandle,
    assistant_conversation_id: &str,
) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT id,state FROM spaces_external_agent_conversations
         WHERE assistant_conversation_id=$1",
    )
    .bind(assistant_conversation_id)
    .fetch_all(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    if rows
        .iter()
        .any(|(_, state)| state == "connected" || state == "connecting")
    {
        return Err("assistant-conversation-busy".into());
    }
    for (id, _) in rows {
        lifecycle::delete(app, &id).await?;
    }
    Ok(())
}

pub(crate) async fn start_for_assistant(
    app: tauri::AppHandle,
    request: StartExternalConversationRequest,
    conversation: Option<(String, String)>,
    updates: tauri::ipc::Channel<crate::spaces::assistant::AssistantTaskUpdate>,
) -> Result<AgentSessionBinding, String> {
    runtime::start(app, request, conversation, Some(updates)).await
}

pub(crate) async fn begin_turn_for_assistant(
    app: tauri::AppHandle,
    session_id: &str,
    prompt: &str,
    attachments: &[crate::spaces::assistant::AssistantMessageAttachment],
) -> Result<String, String> {
    runtime::begin_turn(app, session_id, prompt, attachments).await
}

pub(crate) async fn send_for_assistant(
    app: tauri::AppHandle,
    session_id: &str,
    turn_id: &str,
    prepared: crate::external_agents::prompt::AgentPrompt,
) -> Result<(), String> {
    runtime::send_prepared(app, session_id, turn_id, prepared).await
}

pub(crate) async fn close_for_assistant(
    app: tauri::AppHandle,
    session_id: &str,
) -> Result<(), String> {
    lifecycle::close(app, session_id).await
}

pub(crate) async fn fail_for_assistant(
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
    runtime::mark_disconnected(
        crate::spaces::db::pool(app).await?,
        session_id,
        run_id,
        error,
    )
    .await?;
    if let Some(close_error) = close_error {
        return Err(close_error);
    }
    Ok(())
}

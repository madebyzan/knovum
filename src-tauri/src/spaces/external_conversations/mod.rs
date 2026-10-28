mod contracts;
mod lifecycle;
mod publication;
mod publication_ownership;
mod repository;
mod runtime;
mod staging;
mod updates;

pub(crate) use contracts::{ExternalAgentSessionBinding, StartExternalConversationRequest};
pub(crate) use runtime::AgentTurn;

pub(crate) fn recover(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = lifecycle::recover(app).await {
            log::error!("External Agent conversation recovery failed: {error}");
        }
    });
}

/// Stops every desktop Agent runtime; the app is exiting.
pub(crate) async fn shutdown() {
    runtime::pool::shutdown().await;
}

pub(crate) async fn cancel_by_run(app: &tauri::AppHandle, run_id: &str) -> Result<bool, String> {
    lifecycle::cancel_by_run(app, run_id).await
}

/// Deletes every desktop Agent turn of a Space conversation and its runtime. The workspace belongs
/// to the conversation, which removes it.
pub(crate) async fn purge_for_conversation(
    app: &tauri::AppHandle,
    conversation_id: &str,
) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT id,state FROM spaces_external_agent_conversations WHERE conversation_id=$1",
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
    runtime::pool::close(conversation_id);
    for (id, _) in &rows {
        lifecycle::delete(app, id).await?;
    }
    Ok(())
}

pub(crate) async fn start_for_conversation(
    app: tauri::AppHandle,
    request: StartExternalConversationRequest,
    conversation: Option<(String, String)>,
    updates: tauri::ipc::Channel<crate::spaces::conversations::ConversationTaskUpdate>,
) -> Result<AgentTurn, String> {
    runtime::start(app, request, conversation, updates).await
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
    turn: &mut AgentTurn,
    turn_id: &str,
    prepared: crate::external_agents::prompt::AgentPrompt,
) -> Result<(), String> {
    runtime::send_prepared(app, turn, turn_id, prepared).await
}

/// Publishes what the turn left in the conversation's workspace and settles its Run.
pub(crate) async fn close_for_conversation(
    app: tauri::AppHandle,
    turn: &AgentTurn,
) -> Result<(), String> {
    lifecycle::close(app, &turn.binding.id)
        .await
        .map_err(|error| error.to_string())
}

/// Records a turn that could not complete. The runtime is not touched: a failed turn says
/// nothing about the Agent's next one, and a runtime that did fail has already left the pool.
pub(crate) async fn fail_for_conversation(
    app: &tauri::AppHandle,
    turn: &AgentTurn,
    error: &str,
) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    runtime::lifecycle::mark_disconnected(pool, &turn.binding.id, &turn.binding.run_id, error)
        .await
        .map(|_| ())
}

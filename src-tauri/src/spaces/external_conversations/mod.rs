mod contract;
mod lifecycle;
mod repository;
mod runtime;
mod updates;

pub(crate) use contract::{
    ExternalConversationMessage, ExternalConversationSummary, StartExternalConversationRequest,
};

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

#[tauri::command]
pub async fn start_external_agent_conversation(
    app: tauri::AppHandle,
    request: StartExternalConversationRequest,
) -> Result<ExternalConversationSummary, String> {
    runtime::start(app, request).await
}

#[tauri::command]
pub async fn send_external_agent_message(
    app: tauri::AppHandle,
    conversation_id: String,
    prompt: String,
    attachments: Vec<std::path::PathBuf>,
) -> Result<(), String> {
    runtime::send(app, &conversation_id, &prompt, attachments).await
}

#[tauri::command]
pub async fn set_external_agent_session_option(
    app: tauri::AppHandle,
    conversation_id: String,
    option_id: String,
    value: serde_json::Value,
) -> Result<ExternalConversationSummary, String> {
    runtime::configure(app, &conversation_id, option_id, value).await
}

#[tauri::command]
pub async fn set_external_agent_session_mode(
    app: tauri::AppHandle,
    conversation_id: String,
    mode_id: String,
) -> Result<ExternalConversationSummary, String> {
    runtime::set_mode(app, &conversation_id, mode_id).await
}

#[tauri::command]
pub async fn close_external_agent_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
) -> Result<(), String> {
    lifecycle::close(app, &conversation_id).await
}

#[tauri::command]
pub async fn delete_external_agent_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
) -> Result<(), String> {
    lifecycle::delete(&app, &conversation_id).await
}

#[tauri::command]
pub async fn list_external_agent_conversations(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<ExternalConversationSummary>, String> {
    let pool = crate::spaces::db::pool(&app).await?;
    let conversations = repository::list(pool, &space_id).await?;
    if lifecycle::reconcile(pool, &conversations).await? {
        repository::list(pool, &space_id).await
    } else {
        Ok(conversations)
    }
}

#[tauri::command]
pub async fn list_external_agent_messages(
    app: tauri::AppHandle,
    conversation_id: String,
) -> Result<Vec<ExternalConversationMessage>, String> {
    repository::list_messages(crate::spaces::db::pool(&app).await?, &conversation_id).await
}

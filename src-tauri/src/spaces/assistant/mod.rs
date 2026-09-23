mod contract;
mod repository;
mod task;
mod tools;

pub(crate) fn recover_tasks(app: tauri::AppHandle) {
    task::recover(app);
}

pub(crate) async fn resume_task(app: tauri::AppHandle, run_id: &str) -> Result<(), String> {
    task::resume(app, run_id).await
}

pub(crate) async fn require_paused_checkpoint(
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    task::require_paused_checkpoint(pool, run_id).await
}

pub use contract::{
    AssistantMode, AssistantTaskAccepted, AssistantTaskUpdate, ConversationSummary,
    StartAssistantTaskRequest,
};
pub(crate) use repository::AssistantMessage;

#[tauri::command]
pub async fn start_assistant_task(
    app: tauri::AppHandle,
    request: StartAssistantTaskRequest,
    updates: tauri::ipc::Channel<AssistantTaskUpdate>,
) -> Result<AssistantTaskAccepted, String> {
    task::start(app, request, updates).await
}

pub(crate) fn cancel_task(run_id: &str) -> Result<(), String> {
    task::cancel(run_id)
}

#[tauri::command]
pub async fn list_assistant_conversations(
    app: tauri::AppHandle,
    space_id: String,
    query: Option<String>,
    archived: Option<bool>,
) -> Result<Vec<ConversationSummary>, String> {
    repository::list_conversations(
        crate::spaces::db::pool(&app).await?,
        &space_id,
        query.as_deref(),
        archived.unwrap_or(false),
    )
    .await
}

#[tauri::command]
pub async fn list_assistant_messages(
    app: tauri::AppHandle,
    conversation_id: String,
    branch_id: String,
    before_created_at: Option<i64>,
    before_id: Option<String>,
    limit: Option<u16>,
) -> Result<Vec<repository::AssistantMessage>, String> {
    let before = before_created_at.zip(before_id.as_deref());
    repository::list_messages(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        &branch_id,
        before,
        limit.unwrap_or(100),
    )
    .await
}

#[tauri::command]
pub async fn rename_assistant_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
    title: String,
) -> Result<(), String> {
    repository::rename(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        &title,
    )
    .await
}

#[tauri::command]
pub async fn archive_assistant_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
    archived: bool,
) -> Result<(), String> {
    repository::archive(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        archived,
    )
    .await
}

#[tauri::command]
pub async fn delete_assistant_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
) -> Result<(), String> {
    repository::delete(crate::spaces::db::pool(&app).await?, &conversation_id).await
}

#[tauri::command]
pub async fn export_assistant_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
    branch_id: String,
) -> Result<serde_json::Value, String> {
    let messages = repository::all_messages(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        &branch_id,
    )
    .await?;
    serde_json::to_value(messages).map_err(crate::spaces::db::json_error)
}

#[tauri::command]
pub async fn branch_assistant_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
    source_branch_id: String,
    through_message_id: String,
    include_boundary: bool,
) -> Result<String, String> {
    repository::branch(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        &source_branch_id,
        &through_message_id,
        include_boundary,
    )
    .await
}

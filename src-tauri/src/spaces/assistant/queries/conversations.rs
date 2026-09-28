use super::super::{repository, AssistantConversation};
use crate::command_error::CommandError;

#[tauri::command]
pub async fn list_assistant_conversations(
    app: tauri::AppHandle,
    space_id: String,
    query: Option<String>,
    archived: Option<bool>,
) -> Result<Vec<AssistantConversation>, CommandError> {
    Ok((repository::list_conversations(
        crate::spaces::db::pool(&app).await?,
        &space_id,
        query.as_deref(),
        archived.unwrap_or(false),
    )
    .await)?)
}

#[tauri::command]
pub async fn list_assistant_messages(
    app: tauri::AppHandle,
    conversation_id: String,
    branch_id: String,
    before_created_at: Option<i64>,
    before_id: Option<String>,
    limit: Option<u16>,
) -> Result<Vec<repository::AssistantMessage>, CommandError> {
    let before = before_created_at.zip(before_id.as_deref());
    Ok((repository::list_messages(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        &branch_id,
        before,
        limit.unwrap_or(100),
    )
    .await)?)
}

#[tauri::command]
pub async fn list_assistant_branches(
    app: tauri::AppHandle,
    conversation_id: String,
) -> Result<Vec<String>, CommandError> {
    Ok((repository::list_branches(crate::spaces::db::pool(&app).await?, &conversation_id).await)?)
}

#[tauri::command]
pub async fn load_assistant_branch_messages(
    app: tauri::AppHandle,
    conversation_id: String,
    branch_id: String,
) -> Result<serde_json::Value, CommandError> {
    let messages = repository::all_messages(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        &branch_id,
    )
    .await?;
    Ok((serde_json::to_value(messages).map_err(crate::spaces::db::json_error))?)
}

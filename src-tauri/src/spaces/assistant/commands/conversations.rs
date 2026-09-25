use super::super::{repository, ConversationSummary};

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
    let pool = crate::spaces::db::pool(&app).await?;
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_assistant_tasks t JOIN spaces_runs r ON r.id=t.run_id
         WHERE t.conversation_id=$1 AND r.completed_at IS NULL",
    )
    .bind(&conversation_id)
    .fetch_one(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    if active != 0 {
        return Err("assistant-conversation-busy".into());
    }
    crate::spaces::external_conversations::purge_for_assistant(&app, &conversation_id).await?;
    repository::delete(pool, &conversation_id).await
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

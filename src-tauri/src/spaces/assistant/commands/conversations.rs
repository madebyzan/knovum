use super::super::repository;
use crate::command_error::CommandError;

#[tauri::command]
pub async fn rename_assistant_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
    title: String,
) -> Result<(), CommandError> {
    Ok((repository::rename(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        &title,
    )
    .await)?)
}

#[tauri::command]
pub async fn archive_assistant_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
) -> Result<(), CommandError> {
    Ok((repository::archive(crate::spaces::db::pool(&app).await?, &conversation_id).await)?)
}

#[tauri::command]
pub async fn unarchive_assistant_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
) -> Result<(), CommandError> {
    Ok((repository::unarchive(crate::spaces::db::pool(&app).await?, &conversation_id).await)?)
}

#[tauri::command]
pub async fn delete_assistant_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
) -> Result<(), CommandError> {
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
    Ok((repository::delete(pool, &conversation_id).await)?)
}

#[tauri::command]
pub async fn branch_assistant_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
    source_branch_id: String,
    through_message_id: String,
    include_boundary: bool,
) -> Result<String, CommandError> {
    Ok((repository::branch(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        &source_branch_id,
        &through_message_id,
        include_boundary,
    )
    .await)?)
}

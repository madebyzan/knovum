use super::super::repository;
use crate::app_error::AppError;

#[tauri::command]
pub async fn rename_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
    title: String,
) -> Result<(), AppError> {
    Ok((repository::rename(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        &title,
    )
    .await)?)
}

#[tauri::command]
pub async fn archive_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
) -> Result<(), AppError> {
    Ok((repository::archive(crate::spaces::db::pool(&app).await?, &conversation_id).await)?)
}

#[tauri::command]
pub async fn unarchive_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
) -> Result<(), AppError> {
    Ok((repository::unarchive(crate::spaces::db::pool(&app).await?, &conversation_id).await)?)
}

#[tauri::command]
pub async fn delete_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
) -> Result<(), AppError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_builtin_agent_tasks t JOIN spaces_runs r ON r.id=t.run_id
         WHERE t.conversation_id=$1 AND r.completed_at IS NULL",
    )
    .bind(&conversation_id)
    .fetch_one(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    if active != 0 {
        return Err("conversation-busy".into());
    }
    crate::spaces::external_conversations::purge_for_conversation(&app, &conversation_id).await?;
    Ok((repository::delete(pool, &conversation_id).await)?)
}

#[tauri::command]
pub async fn branch_conversation(
    app: tauri::AppHandle,
    conversation_id: String,
    source_branch_id: String,
    through_message_id: String,
    include_boundary: bool,
) -> Result<String, AppError> {
    Ok((repository::branch(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        &source_branch_id,
        &through_message_id,
        include_boundary,
    )
    .await)?)
}

#[tauri::command]
pub async fn activate_conversation_branch(
    app: tauri::AppHandle,
    conversation_id: String,
    branch_id: String,
) -> Result<(), AppError> {
    Ok((repository::activate_branch(
        crate::spaces::db::pool(&app).await?,
        &conversation_id,
        &branch_id,
    )
    .await)?)
}

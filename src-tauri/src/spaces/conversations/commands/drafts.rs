use crate::{
    app_error::AppError,
    spaces::conversations::{
        contracts::{ConversationDraft, SaveConversationDraftRequest},
        repository,
    },
};

#[tauri::command]
pub async fn load_conversation_draft(
    app: tauri::AppHandle,
    space_id: String,
    conversation_id: String,
) -> Result<Option<ConversationDraft>, AppError> {
    Ok(repository::load_draft(&app, &space_id, &conversation_id).await?)
}

#[tauri::command]
pub async fn save_conversation_draft(
    app: tauri::AppHandle,
    request: SaveConversationDraftRequest,
) -> Result<ConversationDraft, AppError> {
    Ok(repository::save_draft(&app, request).await?)
}

#[tauri::command]
pub async fn remove_conversation_draft(
    app: tauri::AppHandle,
    space_id: String,
    conversation_id: String,
) -> Result<(), AppError> {
    Ok(repository::remove_draft(
        crate::spaces::db::pool(&app).await?,
        &space_id,
        &conversation_id,
    )
    .await?)
}

#[tauri::command]
pub async fn move_conversation_draft(
    app: tauri::AppHandle,
    space_id: String,
    from_conversation_id: String,
    to_conversation_id: String,
) -> Result<(), AppError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let moved = sqlx::query(
        "UPDATE spaces_conversation_drafts SET conversation_id=$1,updated_at=$2
         WHERE space_id=$3 AND conversation_id=$4
           AND EXISTS(SELECT 1 FROM spaces_conversations c JOIN spaces s ON s.id=c.space_id
                      WHERE c.id=$1 AND c.space_id=$3 AND s.archived_at IS NULL)",
    )
    .bind(to_conversation_id)
    .bind(crate::clock::unix_time_millis()?)
    .bind(space_id)
    .bind(from_conversation_id)
    .execute(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    crate::spaces::db::require_one(moved, "conversation-draft-unavailable")?;
    Ok(())
}

use crate::foundation::app_error::AppError;

/// Follows a link in a conversation message to what it names; the window itself never navigates.
#[tauri::command]
pub async fn follow_conversation_link(
    app: tauri::AppHandle,
    message_id: String,
    href: String,
) -> Result<super::super::links::ConversationLink, AppError> {
    super::super::links::follow(&app, &message_id, &href).await
}

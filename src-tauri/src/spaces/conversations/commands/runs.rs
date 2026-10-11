use super::super::{
    builtin_agent, external_agent, ConversationTaskAccepted, ConversationTaskUpdate,
    SubmitConversationTaskRequest,
};
use crate::ai::tasks::TaskTargetSelection;
use crate::foundation::app_error::AppError;

#[tauri::command]
pub async fn start_conversation_task(
    app: tauri::AppHandle,
    request: SubmitConversationTaskRequest,
    updates: tauri::ipc::Channel<ConversationTaskUpdate>,
) -> Result<ConversationTaskAccepted, AppError> {
    match request.target.clone() {
        TaskTargetSelection::Model {
            connection_id,
            model_id,
            temperature,
        } => {
            builtin_agent::start(
                app,
                request.into_model(crate::ai::models::ModelTargetSelection::Model {
                    connection_id,
                    model_id,
                    temperature,
                }),
                updates,
            )
            .await
        }
        TaskTargetSelection::Agent { runtime_kind } => {
            Ok(external_agent::start(app, request, runtime_kind, updates).await?)
        }
    }
}

pub(crate) fn cancel_task(run_id: &str) -> Result<(), String> {
    builtin_agent::cancel(run_id)
}

#[tauri::command]
pub async fn retry_builtin_agent_model(
    app: tauri::AppHandle,
    run_id: String,
    expected_version: i64,
) -> Result<(), AppError> {
    Ok(builtin_agent::retry_model(app, &run_id, expected_version).await?)
}

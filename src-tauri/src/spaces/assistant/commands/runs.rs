use super::super::{
    agent_task, task, AssistantTaskAccepted, AssistantTaskUpdate, SubmitAssistantTaskRequest,
};
use crate::ai::tasks::TaskTargetSelection;
use crate::app_error::AppError;

#[tauri::command]
pub async fn start_assistant_task(
    app: tauri::AppHandle,
    request: SubmitAssistantTaskRequest,
    updates: tauri::ipc::Channel<AssistantTaskUpdate>,
) -> Result<AssistantTaskAccepted, AppError> {
    let accepted = match request.target.clone() {
        TaskTargetSelection::Model {
            connection_id,
            model_id,
            temperature,
        } => {
            task::start(
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
            agent_task::start(app, request, runtime_kind, updates).await
        }
    };
    Ok(accepted?)
}

pub(crate) fn cancel_task(run_id: &str) -> Result<(), String> {
    task::cancel(run_id)
}

#[tauri::command]
pub async fn retry_assistant_model(
    app: tauri::AppHandle,
    run_id: String,
    expected_version: i64,
) -> Result<(), AppError> {
    Ok(task::retry_model(app, &run_id, expected_version).await?)
}

use super::super::{
    agent_task, contract, task, AssistantTaskAccepted, AssistantTaskUpdate,
    SubmitAssistantTaskRequest,
};
use crate::command_error::CommandError;

#[tauri::command]
pub async fn start_assistant_task(
    app: tauri::AppHandle,
    request: SubmitAssistantTaskRequest,
    updates: tauri::ipc::Channel<AssistantTaskUpdate>,
) -> Result<AssistantTaskAccepted, CommandError> {
    let accepted = match request.target.clone() {
        contract::AssistantExecutionTargetSelection::Model {
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
        contract::AssistantExecutionTargetSelection::Agent { runtime_kind } => {
            agent_task::start(app, request, runtime_kind, updates).await
        }
    };
    Ok(accepted?)
}

pub(crate) fn cancel_task(run_id: &str) -> Result<(), String> {
    task::cancel(run_id)
}

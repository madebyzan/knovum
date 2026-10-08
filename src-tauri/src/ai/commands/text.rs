use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    capabilities,
    contracts::ExecutionCapability,
    executions, settings,
    tasks::{
        self,
        completion::{TaskCompletionOutput, TaskInput},
        stream::TaskStreamEvent,
        text::{EditorTransformCommandRequest, TranslationCommandRequest},
        TaskExecutionTarget, TaskTargetSelection,
    },
};
use crate::app_error::AppError;

async fn execute(
    app: &tauri::AppHandle,
    target: TaskTargetSelection,
    input: TaskInput,
    events: Channel<TaskStreamEvent>,
    cancellation: CancellationToken,
) -> Result<TaskCompletionOutput, AppError> {
    let mut target = settings::resolve_task(app, target, ExecutionCapability::TextGeneration)?;
    if matches!(input.purpose, crate::ai::usage::UsagePurpose::Translation) {
        if let TaskExecutionTarget::Model(model) = &mut target {
            model.temperature = 0.0;
        }
    }
    tasks::stream::complete(app, target, input, events, cancellation).await
}

#[tauri::command]
pub async fn transform_editor_text_stream(
    app: tauri::AppHandle,
    request: EditorTransformCommandRequest,
    job_id: String,
    on_event: Channel<TaskStreamEvent>,
) -> Result<TaskCompletionOutput, AppError> {
    let (cancellation, _lease) = executions::acquire(job_id).map_err(AppError::from)?;
    execute(
        &app,
        request.target,
        capabilities::editor::input(request.action, request.text)?,
        on_event,
        cancellation,
    )
    .await
}

#[tauri::command]
pub async fn translate_text_stream(
    app: tauri::AppHandle,
    request: TranslationCommandRequest,
    job_id: String,
    on_event: Channel<TaskStreamEvent>,
) -> Result<TaskCompletionOutput, AppError> {
    let (cancellation, _lease) = executions::acquire(job_id).map_err(AppError::from)?;
    execute(
        &app,
        request.execution_target,
        capabilities::translation::input(request.text, request.target_language)?,
        on_event,
        cancellation,
    )
    .await
}

#[tauri::command]
pub fn cancel_job(job_id: String) -> Result<(), AppError> {
    executions::cancel(&job_id).map_err(AppError::from)
}

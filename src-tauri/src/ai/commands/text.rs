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
        text::{
            EditorTransformCommandRequest, NarrationPreparationCommandRequest,
            TranslationCommandRequest,
        },
        TaskExecutionTarget, TaskTargetSelection,
    },
};
use crate::foundation::app_error::AppError;

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
    let is_model_target = matches!(request.target, TaskTargetSelection::Model { .. });
    let media_capabilities = if is_model_target {
        capabilities::editor::media::EditorMediaCapabilities::detect(&app)
    } else {
        capabilities::editor::media::EditorMediaCapabilities::default()
    };
    let task_input = capabilities::editor::input(&request, media_capabilities)?;
    let mut output = execute(
        &app,
        request.target,
        task_input,
        on_event.clone(),
        cancellation.clone(),
    )
    .await?;
    if media_capabilities.any() {
        output.text = capabilities::editor::media::materialize_generated_media(
            &app,
            &output.text,
            media_capabilities,
            &on_event,
            &cancellation,
        )
        .await?;
    }
    Ok(output)
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
pub async fn prepare_narration_script_stream(
    app: tauri::AppHandle,
    request: NarrationPreparationCommandRequest,
    job_id: String,
    on_event: Channel<TaskStreamEvent>,
) -> Result<TaskCompletionOutput, AppError> {
    let (cancellation, _lease) = executions::acquire(job_id).map_err(AppError::from)?;
    let task_input = capabilities::narration::input(&request)?;
    execute(&app, request.target, task_input, on_event, cancellation).await
}

#[tauri::command]
pub fn cancel_job(job_id: String) -> Result<(), AppError> {
    executions::cancel(&job_id).map_err(AppError::from)
}

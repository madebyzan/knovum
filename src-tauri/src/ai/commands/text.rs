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
            CompletionCommandRequest, EditorTransformCommandRequest, TranslationCommandRequest,
        },
        TaskExecutionTarget, TaskTargetSelection,
    },
};
use crate::app_error::AppError;

async fn execute(
    app: &tauri::AppHandle,
    target: TaskTargetSelection,
    input: TaskInput,
    events: Option<Channel<TaskStreamEvent>>,
    cancellation: CancellationToken,
) -> Result<TaskCompletionOutput, AppError> {
    let mut target = settings::resolve_task(app, target, ExecutionCapability::TextGeneration)?;
    if matches!(input.purpose, crate::ai::usage::UsagePurpose::Translation) {
        if let TaskExecutionTarget::Model(model) = &mut target {
            model.temperature = 0.0;
        }
    }
    if let Some(events) = events {
        tasks::stream::complete(app, target, input, events, cancellation).await
    } else {
        tasks::completion::complete(app, target, input, cancellation).await
    }
}

fn input(request: &CompletionCommandRequest) -> TaskInput {
    TaskInput {
        purpose: crate::ai::usage::UsagePurpose::Knowledge,
        system_prompt: request.system_prompt.clone(),
        instruction: request.instruction.clone(),
        text: request.text.clone(),
        max_tokens: request.max_tokens,
        image: None,
    }
}

#[tauri::command]
pub async fn generate_text(
    app: tauri::AppHandle,
    request: CompletionCommandRequest,
) -> Result<TaskCompletionOutput, AppError> {
    let prepared = input(&request);
    execute(
        &app,
        request.target,
        prepared,
        None,
        CancellationToken::new(),
    )
    .await
}

#[tauri::command]
pub async fn generate_text_stream(
    app: tauri::AppHandle,
    request: CompletionCommandRequest,
    job_id: String,
    on_event: Channel<TaskStreamEvent>,
) -> Result<TaskCompletionOutput, AppError> {
    let (cancellation, _lease) = executions::acquire(job_id).map_err(AppError::from)?;
    let prepared = input(&request);
    execute(&app, request.target, prepared, Some(on_event), cancellation).await
}

#[tauri::command]
pub async fn transform_editor_text(
    app: tauri::AppHandle,
    request: EditorTransformCommandRequest,
) -> Result<TaskCompletionOutput, AppError> {
    execute(
        &app,
        request.target,
        capabilities::editor::input(request.action, request.text)?,
        None,
        CancellationToken::new(),
    )
    .await
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
        Some(on_event),
        cancellation,
    )
    .await
}

#[tauri::command]
pub async fn translate_text(
    app: tauri::AppHandle,
    request: TranslationCommandRequest,
) -> Result<TaskCompletionOutput, AppError> {
    execute(
        &app,
        request.execution_target,
        capabilities::translation::input(request.text, request.target_language)?,
        None,
        CancellationToken::new(),
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
        Some(on_event),
        cancellation,
    )
    .await
}

#[tauri::command]
pub fn cancel_job(job_id: String) -> Result<(), AppError> {
    executions::cancel(&job_id).map_err(AppError::from)
}

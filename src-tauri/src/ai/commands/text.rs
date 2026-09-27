use tauri::ipc::Channel;

use crate::ai::{
    capabilities, contracts,
    error::AiError,
    executions,
    models::{self, CompletionOutput, CompletionStreamEvent},
    settings,
};

fn resolve_target(
    app: &tauri::AppHandle,
    target: models::ExecutionTargetSelection,
) -> Result<models::ExecutionTarget, AiError> {
    settings::resolve(app, target, contracts::ExecutionCapability::TextGeneration)
}

#[tauri::command]
pub async fn generate_text(
    app: tauri::AppHandle,
    request: models::CompletionCommandRequest,
) -> Result<CompletionOutput, AiError> {
    capabilities::chat::complete(models::CompletionRequest {
        purpose: crate::ai::usage::UsagePurpose::Knowledge,
        target: resolve_target(&app, request.target)?,
        system_prompt: request.system_prompt,
        instruction: request.instruction,
        text: request.text,
        max_tokens: request.max_tokens,
    })
    .await
}

#[tauri::command]
pub async fn generate_text_stream(
    app: tauri::AppHandle,
    request: models::CompletionCommandRequest,
    job_id: String,
    on_event: Channel<CompletionStreamEvent>,
) -> Result<CompletionOutput, AiError> {
    let (cancellation, _lease) = executions::acquire(job_id).map_err(AiError::from)?;
    capabilities::chat::complete_stream(
        models::CompletionRequest {
            purpose: crate::ai::usage::UsagePurpose::Knowledge,
            target: resolve_target(&app, request.target)?,
            system_prompt: request.system_prompt,
            instruction: request.instruction,
            text: request.text,
            max_tokens: request.max_tokens,
        },
        on_event,
        cancellation,
    )
    .await
}

#[tauri::command]
pub async fn transform_editor_text(
    app: tauri::AppHandle,
    request: models::EditorTransformCommandRequest,
) -> Result<CompletionOutput, AiError> {
    capabilities::editor::transform(models::EditorTransformRequest {
        target: resolve_target(&app, request.target)?,
        action: request.action,
        text: request.text,
    })
    .await
}

#[tauri::command]
pub async fn transform_editor_text_stream(
    app: tauri::AppHandle,
    request: models::EditorTransformCommandRequest,
    job_id: String,
    on_event: Channel<CompletionStreamEvent>,
) -> Result<CompletionOutput, AiError> {
    let (cancellation, _lease) = executions::acquire(job_id).map_err(AiError::from)?;
    capabilities::editor::transform_stream(
        models::EditorTransformRequest {
            target: resolve_target(&app, request.target)?,
            action: request.action,
            text: request.text,
        },
        on_event,
        cancellation,
    )
    .await
}

#[tauri::command]
pub async fn translate_text(
    app: tauri::AppHandle,
    request: models::TranslationCommandRequest,
) -> Result<CompletionOutput, AiError> {
    capabilities::translation::translate(models::TranslationRequest {
        execution_target: resolve_target(&app, request.execution_target)?,
        text: request.text,
        target_language: request.target_language,
    })
    .await
}

#[tauri::command]
pub async fn translate_text_stream(
    app: tauri::AppHandle,
    request: models::TranslationCommandRequest,
    job_id: String,
    on_event: Channel<CompletionStreamEvent>,
) -> Result<CompletionOutput, AiError> {
    let (cancellation, _lease) = executions::acquire(job_id).map_err(AiError::from)?;
    capabilities::translation::translate_stream(
        models::TranslationRequest {
            execution_target: resolve_target(&app, request.execution_target)?,
            text: request.text,
            target_language: request.target_language,
        },
        on_event,
        cancellation,
    )
    .await
}

#[tauri::command]
pub fn cancel_job(job_id: String) -> Result<(), AiError> {
    executions::cancel(&job_id).map_err(AiError::from)
}

use tauri::AppHandle;

use super::{image_layers, video_edit, CompletedDerivationOutput};
use crate::app_error::AppError;

#[tauri::command]
pub async fn generate_image_layers(
    app: AppHandle,
    request: image_layers::GenerateImageLayersRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<Vec<CompletedDerivationOutput>, AppError> {
    image_layers::generate(app, request, on_stage).await
}

#[tauri::command]
pub fn cancel_image_layer_generation(generation_id: String) -> Result<(), AppError> {
    Ok((image_layers::cancel(&generation_id))?)
}

#[tauri::command]
pub async fn render_video_edit(
    app: AppHandle,
    request: video_edit::RenderVideoEditRequest,
    on_progress: tauri::ipc::Channel<video_edit::VideoEditProgress>,
) -> Result<CompletedDerivationOutput, AppError> {
    video_edit::render(app, request, on_progress).await
}

#[tauri::command]
pub async fn cancel_video_edit_render(derivation_id: String) -> Result<(), AppError> {
    Ok((video_edit::cancel(&derivation_id))?)
}

#[tauri::command]
pub async fn prepare_video_edit_proxy(
    app: AppHandle,
    request: video_edit::PrepareVideoEditProxyRequest,
) -> Result<String, AppError> {
    Ok((video_edit::prepare_proxy(app, request).await)?)
}

#[tauri::command]
pub async fn cancel_video_edit_proxy(job_id: String) -> Result<(), AppError> {
    Ok((video_edit::cancel_proxy(&job_id))?)
}

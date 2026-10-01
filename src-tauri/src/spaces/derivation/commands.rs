use crate::command_error::CommandError;
use tauri::AppHandle;

use super::{image_cutout, image_layers, image_operations, video_edit, CompletedDerivationOutput};

#[tauri::command]
pub async fn generate_image_layers(
    app: AppHandle,
    request: image_layers::GenerateImageLayersRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<Vec<CompletedDerivationOutput>, CommandError> {
    Ok((image_layers::generate(app, request, on_stage).await)?)
}

#[tauri::command]
pub fn cancel_image_layer_generation(generation_id: String) -> Result<(), CommandError> {
    Ok((image_layers::cancel(&generation_id))?)
}

#[tauri::command]
pub async fn derive_image(
    app: AppHandle,
    request: image_operations::DeriveImageRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<CompletedDerivationOutput, CommandError> {
    Ok((image_operations::derive(app, request, on_stage).await)?)
}

#[tauri::command]
pub async fn run_image_cutout_derivation(
    app: AppHandle,
    request: image_cutout::StartImageCutoutRequest,
) -> Result<CompletedDerivationOutput, CommandError> {
    Ok(image_cutout::run(app, request).await?)
}

#[tauri::command]
pub async fn render_video_edit(
    app: AppHandle,
    request: video_edit::RenderVideoEditRequest,
    on_progress: tauri::ipc::Channel<video_edit::VideoEditProgress>,
) -> Result<CompletedDerivationOutput, CommandError> {
    video_edit::render(app, request, on_progress)
        .await
        .map_err(|message| {
            if message == video_edit::CANCELLED {
                CommandError::new("video_edit_render_cancelled", message)
            } else {
                message.into()
            }
        })
}

#[tauri::command]
pub async fn cancel_video_edit_render(derivation_id: String) -> Result<(), CommandError> {
    Ok((video_edit::cancel(&derivation_id))?)
}

#[tauri::command]
pub async fn prepare_video_edit_proxy(
    app: AppHandle,
    request: video_edit::PrepareVideoEditProxyRequest,
) -> Result<String, CommandError> {
    Ok((video_edit::prepare_proxy(app, request).await)?)
}

#[tauri::command]
pub async fn cancel_video_edit_proxy(job_id: String) -> Result<(), CommandError> {
    Ok((video_edit::cancel_proxy(&job_id))?)
}

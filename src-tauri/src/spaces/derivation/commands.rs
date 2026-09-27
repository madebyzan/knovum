use crate::command_error::CommandError;
use tauri::AppHandle;

use super::{
    document_export, grounded_document, image_cutout, image_layers, image_operations,
    subtitle_extraction, subtitled_video, video_edit, video_generation, visualization,
    CompletedDerivationOutput, DerivationRelation,
};

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
pub async fn start_image_cutout_derivation(
    app: AppHandle,
    request: image_cutout::StartImageCutoutRequest,
) -> Result<(), CommandError> {
    Ok((image_cutout::start(&app, request).await)?)
}

#[tauri::command]
pub async fn submit_image_cutout_derivation(
    app: AppHandle,
    request: image_cutout::CompleteImageCutoutRequest,
) -> Result<CompletedDerivationOutput, CommandError> {
    Ok((image_cutout::complete(app, request).await)?)
}

#[tauri::command]
pub async fn fail_image_cutout_derivation(
    app: AppHandle,
    request: image_cutout::FailImageCutoutRequest,
) -> Result<(), CommandError> {
    Ok((image_cutout::fail(&app, request).await)?)
}

#[tauri::command]
pub async fn render_subtitled_video(
    app: AppHandle,
    request: subtitled_video::RenderSubtitledVideoRequest,
    on_progress: tauri::ipc::Channel<subtitled_video::SubtitledVideoProgress>,
) -> Result<CompletedDerivationOutput, CommandError> {
    Ok((subtitled_video::render(app, request, on_progress).await)?)
}

#[tauri::command]
pub async fn create_subtitle_resource(
    app: AppHandle,
    request: subtitle_extraction::CreateSubtitleResourceRequest,
) -> Result<DerivationRelation, CommandError> {
    Ok((subtitle_extraction::create(&app, request).await)?)
}

#[tauri::command]
pub async fn cancel_subtitled_video_render(derivation_id: String) -> Result<(), CommandError> {
    Ok((subtitled_video::cancel(derivation_id).await)?)
}

#[tauri::command]
pub async fn render_video_edit(
    app: AppHandle,
    request: video_edit::RenderVideoEditRequest,
    on_progress: tauri::ipc::Channel<video_edit::VideoEditProgress>,
) -> Result<CompletedDerivationOutput, CommandError> {
    Ok((video_edit::render(app, request, on_progress).await)?)
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

#[tauri::command]
pub async fn generate_grounded_document(
    app: AppHandle,
    request: grounded_document::GenerateGroundedDocumentRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<grounded_document::GroundedDocumentOutput, CommandError> {
    Ok((grounded_document::generate(app, request, on_stage).await)?)
}

#[tauri::command]
pub async fn generate_visualization(
    app: AppHandle,
    request: visualization::GenerateVisualizationRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<visualization::VisualizationOutput, CommandError> {
    Ok((visualization::generate(app, request, on_stage).await)?)
}

#[tauri::command]
pub async fn generate_video(
    app: AppHandle,
    request: video_generation::GenerateVideoRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<video_generation::VideoOutput, CommandError> {
    Ok((video_generation::generate(app, request, on_stage).await)?)
}

#[tauri::command]
pub async fn cancel_video_generation(derivation_id: String) -> Result<(), CommandError> {
    Ok((video_generation::registry::cancel(&derivation_id))?)
}

#[tauri::command]
pub async fn export_document_as_docx(
    app: AppHandle,
    request: document_export::ExportDocumentRequest,
) -> Result<document_export::ExportDocumentOutput, CommandError> {
    Ok((document_export::export(&app, request, document_export::DOCX).await)?)
}

#[tauri::command]
pub async fn export_document_as_pdf(
    app: AppHandle,
    request: document_export::ExportDocumentRequest,
) -> Result<document_export::ExportDocumentOutput, CommandError> {
    Ok((document_export::export(&app, request, document_export::PDF).await)?)
}

#[tauri::command]
pub async fn export_document_as_txt(
    app: AppHandle,
    request: document_export::ExportDocumentRequest,
) -> Result<document_export::ExportDocumentOutput, CommandError> {
    Ok((document_export::export(&app, request, document_export::TEXT).await)?)
}

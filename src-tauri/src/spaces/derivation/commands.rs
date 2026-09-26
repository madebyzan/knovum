use crate::command_error::CommandError;
use tauri::AppHandle;

use super::{
    document_export, grounded_document, image_cutout, image_layers, image_operations,
    subtitle_extraction, subtitled_video, video_edit, video_generation, visualization,
    CompletedDerivationOutput, DerivationRelation,
};

#[tauri::command]
pub async fn get_image_layer_capability() -> image_layers::ImageLayerCapability {
    image_layers::capability().await
}

#[tauri::command]
pub async fn create_image_layers(
    app: AppHandle,
    request: image_layers::CreateImageLayersRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<Vec<CompletedDerivationOutput>, CommandError> {
    Ok((image_layers::create(app, request, on_stage).await)?)
}

#[tauri::command]
pub fn cancel_image_layers(id: String) -> Result<(), CommandError> {
    Ok((image_layers::cancel(&id))?)
}

#[tauri::command]
pub async fn get_image_operation_capabilities(
    app: AppHandle,
) -> image_operations::ImageOperationCapabilities {
    image_operations::capabilities(&app).await
}

#[tauri::command]
pub async fn create_image_operation(
    app: AppHandle,
    request: image_operations::CreateImageOperationRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<CompletedDerivationOutput, CommandError> {
    Ok((image_operations::create(app, request, on_stage).await)?)
}

#[tauri::command]
pub async fn check_image_cutout_source(
    app: AppHandle,
    request: image_cutout::CheckImageCutoutSourceRequest,
) -> Result<bool, CommandError> {
    Ok((image_cutout::check_source(&app, request).await)?)
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
pub async fn create_subtitled_video(
    app: AppHandle,
    request: subtitled_video::CreateSubtitledVideoRequest,
    on_progress: tauri::ipc::Channel<subtitled_video::SubtitledVideoProgress>,
) -> Result<CompletedDerivationOutput, CommandError> {
    Ok((subtitled_video::create(app, request, on_progress).await)?)
}

#[tauri::command]
pub async fn create_subtitle_resource(
    app: AppHandle,
    request: subtitle_extraction::CreateSubtitleResourceRequest,
) -> Result<DerivationRelation, CommandError> {
    Ok((subtitle_extraction::create(&app, request).await)?)
}

#[tauri::command]
pub async fn cancel_subtitled_video(job_id: String) -> Result<(), CommandError> {
    Ok((subtitled_video::cancel(job_id).await)?)
}

#[tauri::command]
pub async fn render_video_edit(
    app: AppHandle,
    request: video_edit::RenderVideoEditRequest,
    on_progress: tauri::ipc::Channel<video_edit::VideoEditProgress>,
) -> Result<CompletedDerivationOutput, CommandError> {
    Ok((video_edit::create(app, request, on_progress).await)?)
}

#[tauri::command]
pub async fn cancel_video_edit_render(job_id: String) -> Result<(), CommandError> {
    Ok((video_edit::cancel(&job_id))?)
}

#[tauri::command]
pub async fn prepare_video_edit_proxy(
    app: AppHandle,
    request: video_edit::PrepareRequest,
) -> Result<String, CommandError> {
    Ok((video_edit::prepare_proxy(app, request).await)?)
}

#[tauri::command]
pub async fn cancel_video_edit_proxy(job_id: String) -> Result<(), CommandError> {
    Ok((video_edit::cancel_proxy(&job_id))?)
}

#[tauri::command]
pub async fn list_video_fonts() -> Result<Vec<video_edit::fonts::InstalledFont>, CommandError> {
    Ok(crate::blocking::run(|| Ok::<_, String>(video_edit::fonts::installed())).await?)
}

#[tauri::command]
pub async fn create_document(
    app: AppHandle,
    request: grounded_document::CreateGroundedDocumentRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<grounded_document::GroundedDocumentOutput, CommandError> {
    Ok((grounded_document::create(app, request, on_stage).await)?)
}

#[tauri::command]
pub async fn create_visualization(
    app: AppHandle,
    request: visualization::CreateVisualizationRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<visualization::VisualizationOutput, CommandError> {
    Ok((visualization::create(app, request, on_stage).await)?)
}

#[tauri::command]
pub async fn create_video(
    app: AppHandle,
    request: video_generation::CreateVideoRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<video_generation::VideoOutput, CommandError> {
    Ok((video_generation::create(app, request, on_stage).await)?)
}

#[tauri::command]
pub async fn cancel_video_generation(job_id: String) -> Result<(), CommandError> {
    Ok((video_generation::registry::cancel(&job_id))?)
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

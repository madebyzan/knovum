use tauri::AppHandle;

use super::{
    document_export, grounded_document, image_cutout, image_layers, image_operations,
    presentation_export, subtitle_extraction, subtitled_video, video_edit, video_generation,
    visualization, CompletedDerivationOutput, DerivationRelation,
};

#[tauri::command]
pub fn get_image_layer_capability() -> image_layers::ImageLayerCapability {
    image_layers::capability()
}

#[tauri::command]
pub async fn create_image_layers(
    app: AppHandle,
    request: image_layers::CreateImageLayersRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<Vec<CompletedDerivationOutput>, String> {
    image_layers::create(app, request, on_stage).await
}

#[tauri::command]
pub fn cancel_image_layers(id: String) -> Result<(), String> {
    image_layers::cancel(&id)
}

#[tauri::command]
pub fn get_image_operation_capabilities(
    app: AppHandle,
) -> image_operations::ImageOperationCapabilities {
    image_operations::capabilities(&app)
}

#[tauri::command]
pub async fn create_image_operation(
    app: AppHandle,
    request: image_operations::CreateImageOperationRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<CompletedDerivationOutput, String> {
    image_operations::create(app, request, on_stage).await
}

#[tauri::command]
pub async fn start_image_cutout_derivation(
    app: AppHandle,
    request: image_cutout::StartImageCutoutRequest,
) -> Result<(), String> {
    image_cutout::start(&app, request).await
}

#[tauri::command]
pub async fn submit_image_cutout_derivation(
    app: AppHandle,
    request: image_cutout::CompleteImageCutoutRequest,
) -> Result<CompletedDerivationOutput, String> {
    image_cutout::complete(app, request).await
}

#[tauri::command]
pub async fn fail_image_cutout_derivation(
    app: AppHandle,
    request: image_cutout::FailImageCutoutRequest,
) -> Result<(), String> {
    image_cutout::fail(&app, request).await
}

#[tauri::command]
pub async fn create_subtitled_video(
    app: AppHandle,
    request: subtitled_video::CreateSubtitledVideoRequest,
    on_progress: tauri::ipc::Channel<subtitled_video::SubtitledVideoProgress>,
) -> Result<CompletedDerivationOutput, String> {
    subtitled_video::create(app, request, on_progress).await
}

#[tauri::command]
pub async fn create_subtitle_resource(
    app: AppHandle,
    request: subtitle_extraction::CreateSubtitleResourceRequest,
) -> Result<DerivationRelation, String> {
    subtitle_extraction::create(&app, request).await
}

#[tauri::command]
pub async fn cancel_subtitled_video(job_id: String) -> Result<(), String> {
    subtitled_video::cancel(job_id).await
}

#[tauri::command]
pub async fn render_video_edit(
    app: AppHandle,
    request: video_edit::RenderVideoEditRequest,
    on_progress: tauri::ipc::Channel<video_edit::VideoEditProgress>,
) -> Result<CompletedDerivationOutput, String> {
    video_edit::create(app, request, on_progress).await
}

#[tauri::command]
pub async fn cancel_video_edit_render(job_id: String) -> Result<(), String> {
    video_edit::cancel(&job_id)
}

#[tauri::command]
pub async fn prepare_video_edit_proxy(
    app: AppHandle,
    request: video_edit::PrepareRequest,
) -> Result<String, String> {
    video_edit::prepare_proxy(app, request).await
}

#[tauri::command]
pub async fn cancel_video_edit_proxy(job_id: String) -> Result<(), String> {
    video_edit::cancel_proxy(&job_id)
}

#[tauri::command]
pub async fn create_document(
    app: AppHandle,
    request: grounded_document::CreateGroundedDocumentRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<grounded_document::GroundedDocumentOutput, String> {
    grounded_document::create(app, request, on_stage).await
}

#[tauri::command]
pub async fn create_visualization(
    app: AppHandle,
    request: visualization::CreateVisualizationRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<visualization::VisualizationOutput, String> {
    visualization::create(app, request, on_stage).await
}

#[tauri::command]
pub async fn submit_presentation(
    app: AppHandle,
    request: presentation_export::CompletePresentationRequest,
) -> Result<presentation_export::PresentationOutput, String> {
    presentation_export::complete(&app, request).await
}

#[tauri::command]
pub async fn create_video(
    app: AppHandle,
    request: video_generation::CreateVideoRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<video_generation::VideoOutput, String> {
    video_generation::create(app, request, on_stage).await
}

#[tauri::command]
pub async fn cancel_video_generation(job_id: String) -> Result<(), String> {
    video_generation::registry::cancel(&job_id)
}

#[tauri::command]
pub async fn export_document_as_docx(
    app: AppHandle,
    request: document_export::ExportDocumentRequest,
) -> Result<document_export::ExportDocumentOutput, String> {
    document_export::docx::export(&app, request).await
}

#[tauri::command]
pub async fn export_document_as_pdf(
    app: AppHandle,
    request: document_export::ExportDocumentRequest,
) -> Result<document_export::ExportDocumentOutput, String> {
    document_export::pdf::export(&app, request).await
}

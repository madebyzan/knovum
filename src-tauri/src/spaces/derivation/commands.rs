use tauri::AppHandle;

use super::{
    document_export, grounded_document, image_cutout, knowledge_video, presentation_export,
    subtitle_extraction, subtitled_video, visualization, CompletedDerivationOutput,
    DerivationRelation,
};

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
pub async fn create_grounded_document(
    app: AppHandle,
    request: grounded_document::CreateGroundedDocumentRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<grounded_document::GroundedDocumentOutput, String> {
    grounded_document::create(app, request, on_stage).await
}

#[tauri::command]
pub async fn create_knowledge_visualization(
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
pub async fn create_knowledge_video(
    app: AppHandle,
    request: knowledge_video::CreateVideoRequest,
    on_stage: tauri::ipc::Channel<String>,
) -> Result<knowledge_video::VideoOutput, String> {
    knowledge_video::create(app, request, on_stage).await
}

#[tauri::command]
pub async fn cancel_knowledge_video(job_id: String) -> Result<(), String> {
    knowledge_video::registry::cancel(&job_id)
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

use tauri::AppHandle;

pub(crate) fn reserve(app: &AppHandle, prefix: &str, extension: &str) -> Result<String, String> {
    Ok(crate::spaces::assets::asset_directory(app)?
        .join(format!("{prefix}-{}.{}", uuid::Uuid::new_v4(), extension))
        .to_string_lossy()
        .into_owned())
}

#[tauri::command]
pub fn allocate_visualization_output_path(app: AppHandle) -> Result<String, String> {
    reserve(&app, "generated", "png")
}

#[tauri::command]
pub fn allocate_document_docx_path(app: AppHandle) -> Result<String, String> {
    reserve(&app, "document", "docx")
}

#[tauri::command]
pub fn allocate_document_pdf_path(app: AppHandle) -> Result<String, String> {
    reserve(&app, "document", "pdf")
}

#[tauri::command]
pub fn allocate_presentation_pptx_path(app: AppHandle) -> Result<String, String> {
    reserve(&app, "presentation", "pptx")
}

#[tauri::command]
pub fn allocate_video_output_path(app: AppHandle) -> Result<String, String> {
    reserve(&app, "generated", "mp4")
}

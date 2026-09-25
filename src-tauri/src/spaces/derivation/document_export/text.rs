use tauri::AppHandle;

use super::{execute, ExportDocumentOutput, ExportDocumentRequest};

pub async fn export(
    app: &AppHandle,
    request: ExportDocumentRequest,
) -> Result<ExportDocumentOutput, String> {
    execute::export(app, request, execute::TEXT).await
}

use tauri::AppHandle;

use super::lifecycle::{self, ExportDocumentOutput, ExportDocumentRequest};

pub async fn export(
    app: &AppHandle,
    request: ExportDocumentRequest,
) -> Result<ExportDocumentOutput, String> {
    let prepared = lifecycle::prepare(
        app,
        &request,
        "export-document-as-docx",
        "docx-rs:0.4.22",
        "docx",
    )
    .await?;
    let path = prepared.path.clone();
    let exported = async {
        let bytes = tokio::task::spawn_blocking(move || {
            crate::documents::generation::docx::render(&prepared.markdown)
        })
        .await
        .map_err(|error| error.to_string())??;
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|error| error.to_string())?;
        lifecycle::complete(app, &request, &bytes, "DOCX exported from current Markdown").await
    }
    .await;
    match exported {
        Ok(output) => Ok(output),
        Err(error) => Err(lifecycle::handle_failure(app, &request, &path, &error).await),
    }
}

use tauri::AppHandle;

use super::lifecycle::{self, ExportDocumentOutput, ExportDocumentRequest};

pub async fn export(
    app: &AppHandle,
    request: ExportDocumentRequest,
) -> Result<ExportDocumentOutput, String> {
    let prepared =
        lifecycle::prepare(app, &request, "markdown-pdf-export", "typst:0.15.1", "pdf").await?;
    let path = prepared.path.clone();
    let exported = async {
        let bytes = tokio::task::spawn_blocking(move || {
            crate::documents::generation::pdf::render(&prepared.markdown)
        })
        .await
        .map_err(|error| error.to_string())??;
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|error| error.to_string())?;
        lifecycle::complete(app, &request, &bytes, "PDF exported from current Markdown").await
    }
    .await;
    match exported {
        Ok(output) => Ok(output),
        Err(error) => Err(lifecycle::handle_failure(app, &request, &path, &error).await),
    }
}

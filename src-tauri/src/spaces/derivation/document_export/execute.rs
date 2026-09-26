use tauri::AppHandle;

use super::lifecycle::{self, ExportDocumentOutput, ExportDocumentRequest};
use crate::documents::generation::{self, document::DocumentSource};

type Renderer = fn(&DocumentSource) -> Result<Vec<u8>, String>;

pub(in crate::spaces::derivation) struct Format {
    pub action: &'static str,
    pub engine: &'static str,
    pub extension: &'static str,
    pub rationale: &'static str,
    pub render: Renderer,
}

pub(in crate::spaces::derivation) const DOCX: Format = Format {
    action: "export-document-as-docx",
    engine: "docx-rs:0.4.22;knovum-document:2",
    extension: "docx",
    rationale: "DOCX exported from current Markdown",
    render: generation::docx::render,
};
pub(in crate::spaces::derivation) const PDF: Format = Format {
    action: "export-document-as-pdf",
    engine: "typst:0.15.1;knovum-document:2",
    extension: "pdf",
    rationale: "PDF exported from current Markdown",
    render: generation::pdf::render,
};
pub(in crate::spaces::derivation) const TEXT: Format = Format {
    action: "export-document-as-txt",
    engine: "knovum-document:2",
    extension: "txt",
    rationale: "Plain text exported from current Markdown",
    render: generation::render_text,
};

pub(in crate::spaces::derivation) async fn export(
    app: &AppHandle,
    request: ExportDocumentRequest,
    format: Format,
) -> Result<ExportDocumentOutput, String> {
    let prepared = lifecycle::prepare(
        app,
        &request,
        format.action,
        format.engine,
        format.extension,
    )
    .await?;
    let path = prepared.path.clone();
    let renderer = format.render;
    let exported = async {
        let bytes = tokio::task::spawn_blocking(move || renderer(&prepared.source))
            .await
            .map_err(|error| error.to_string())??;
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|error| error.to_string())?;
        lifecycle::complete(app, &request, &bytes, format.rationale).await
    }
    .await;
    match exported {
        Ok(output) => Ok(output),
        Err(error) => Err(lifecycle::handle_failure(app, &request, &path, &error).await),
    }
}

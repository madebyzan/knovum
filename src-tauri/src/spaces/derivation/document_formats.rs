use crate::documents::generation::{self, DocumentFormat as RendererFormat};

pub(super) struct DocumentFormat {
    pub extension: &'static str,
    pub engine: &'static str,
    pub kind: RendererFormat,
}
pub(super) fn require(action: &str) -> Result<DocumentFormat, String> {
    let (extension, engine, kind) = match action {
        "render-presentation" => ("pptx", generation::pptx::ENGINE, RendererFormat::Pptx),
        "export-document-as-docx" => (
            "docx",
            "docx-rs:0.4.22;knovum-document:2",
            RendererFormat::Docx,
        ),
        "export-document-as-pdf" => ("pdf", "typst:0.15.1;knovum-document:2", RendererFormat::Pdf),
        "export-document-as-txt" => ("txt", "knovum-document:2", RendererFormat::Text),
        _ => return Err("action-input-invalid: unsupported document format".into()),
    };
    Ok(DocumentFormat {
        extension,
        engine,
        kind,
    })
}

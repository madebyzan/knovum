use crate::documents::generation::{self, document::DocumentSource};
type DocumentRenderer = fn(&DocumentSource) -> Result<Vec<u8>, String>;

pub(super) struct DocumentFormat {
    pub extension: &'static str,
    pub engine: &'static str,
    pub render: DocumentRenderer,
}
pub(super) fn require(action: &str) -> Result<DocumentFormat, String> {
    let (extension, engine, render): (_, _, DocumentRenderer) = match action {
        "render-presentation" => ("pptx", generation::pptx::ENGINE, generation::pptx::render),
        "export-document-as-docx" => (
            "docx",
            "docx-rs:0.4.22;knovum-document:2",
            generation::docx::render,
        ),
        "export-document-as-pdf" => (
            "pdf",
            "typst:0.15.1;knovum-document:2",
            generation::pdf::render,
        ),
        "export-document-as-txt" => ("txt", "knovum-document:2", generation::render_text),
        _ => return Err("action-input-invalid: unsupported document format".into()),
    };
    Ok(DocumentFormat {
        extension,
        engine,
        render,
    })
}

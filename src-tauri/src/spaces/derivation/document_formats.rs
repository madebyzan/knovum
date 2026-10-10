use crate::documents::generation::{self, DocumentFormat};

pub(super) struct DocumentConversionTarget {
    pub extension: &'static str,
    pub engine: &'static str,
    pub format: DocumentFormat,
}

impl DocumentConversionTarget {
    pub(super) fn engine_for_source(&self, request: &serde_json::Value) -> Result<String, String> {
        match request
            .pointer("/document/kind")
            .and_then(serde_json::Value::as_str)
        {
            Some("markdown") => Ok(self.engine.into()),
            Some("file") => {
                let path = request
                    .pointer("/document/data/assetPath")
                    .and_then(serde_json::Value::as_str)
                    .ok_or("Document source path is required")?;
                let extension = std::path::Path::new(path).extension();
                if extension.is_some_and(|extension| extension.eq_ignore_ascii_case("pdf")) {
                    return Ok(format!("pdf-inspector:1.25.2;{}", self.engine));
                }
                if matches!(
                    self.format,
                    DocumentFormat::Docx
                        | DocumentFormat::Pptx
                        | DocumentFormat::Html
                        | DocumentFormat::Text
                ) {
                    return Ok("office_oxide:0.1.13;knovum-office-conversion:1".into());
                }
                Ok(format!("office_oxide:0.1.13;{}", self.engine))
            }
            _ => Err("Document source kind is required".into()),
        }
    }
}
pub(super) fn require(action_id: &str) -> Result<DocumentConversionTarget, String> {
    let (extension, engine, format) = match action_id {
        "render-presentation" => ("pptx", generation::pptx::ENGINE, DocumentFormat::Pptx),
        "convert-document-to-docx" => (
            "docx",
            "docx-rs:0.4.22;knovum-document:2",
            DocumentFormat::Docx,
        ),
        "convert-document-to-pdf" => ("pdf", "typst:0.15.1;knovum-document:2", DocumentFormat::Pdf),
        "convert-document-to-txt" => ("txt", "knovum-document:2", DocumentFormat::Text),
        "convert-document-to-html" => (
            "html",
            "pulldown-cmark:0.13.4;knovum-document-html:1",
            DocumentFormat::Html,
        ),
        _ => return Err("action-input-invalid: unsupported document format".into()),
    };
    Ok(DocumentConversionTarget {
        extension,
        engine,
        format,
    })
}

pub(super) fn resolve_source_title(
    kind: &str,
    content: &serde_json::Value,
) -> Result<String, String> {
    let title = content
        .get("title")
        .and_then(serde_json::Value::as_str)
        .ok_or("Document source title is required")?;
    if kind == "file" && content.get("fileName").and_then(serde_json::Value::as_str) == Some(title)
    {
        return std::path::Path::new(title)
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .map(str::to_owned)
            .ok_or_else(|| "Document source filename is invalid".into());
    }
    Ok(title.into())
}

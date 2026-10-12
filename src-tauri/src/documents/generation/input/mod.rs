pub(crate) mod file;
pub(super) mod note;
pub(super) mod office;
mod pdf;

use std::path::PathBuf;

use super::{document::DocumentSource, DocumentFormat};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub(crate) enum DocumentInput {
    Markdown(DocumentSource),
    Note(super::document::NoteSource),
    File(DocumentFileSource),
    Pdf(pdf::PdfSource),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DocumentFileSource {
    pub title: String,
    pub path: PathBuf,
    pub source_revision: String,
}

impl DocumentInput {
    pub(super) fn validate(&self) -> Result<(), String> {
        match self {
            Self::Markdown(source) => super::markdown::validate_source(source),
            Self::Note(source) => super::markdown::validate_source(&DocumentSource {
                title: source.title.clone(),
                markdown: String::new(),
                citations: vec![],
                images: vec![],
            }),
            Self::File(source) => source.validate(),
            Self::Pdf(source) => source.validate(),
        }
    }

    pub(crate) fn snapshots(&self) -> Vec<DocumentFileSource> {
        match self {
            Self::Markdown(source) => source
                .images
                .iter()
                .map(|image| image.file.clone())
                .collect(),
            Self::Note(source) => source
                .images
                .iter()
                .map(|image| image.file.clone())
                .collect(),
            Self::File(source) => vec![source.clone()],
            Self::Pdf(source) => vec![source.file.clone()],
        }
    }

    pub(super) fn render(self, format: DocumentFormat) -> Result<Vec<u8>, String> {
        match self {
            Self::Markdown(source) => format.render(&source),
            Self::Note(source) => {
                let ir = note::parse(&source, format)?;
                super::structured::render(ir, format, &source.title)
            }
            Self::File(source) => file::render(&source, format),
            Self::Pdf(source) => pdf::render(&source, format),
        }
    }
}

pub(crate) use pdf::{PdfSource, RecognizedPage};

impl DocumentFileSource {
    fn validate(&self) -> Result<(), String> {
        if !self.path.is_absolute() || self.source_revision.is_empty() {
            return Err("Document source path and revision are required".into());
        }
        if !is_supported_source_format(&self.path) {
            return Err("Unsupported document source format".into());
        }
        super::markdown::validate_source(&DocumentSource {
            title: self.title.clone(),
            ..Default::default()
        })
    }

    pub(crate) fn verify_current(&self) -> Result<(), String> {
        file::read(self).map(|_| ())
    }
}

pub(crate) fn is_supported_source_format(path: &std::path::Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
        || office_oxide::DocumentFormat::from_path(path).is_some()
}

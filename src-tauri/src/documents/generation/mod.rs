pub(crate) mod document;
pub(crate) mod docx;
mod html;
pub(crate) mod images;
pub(crate) mod input;
mod links;
mod markdown;
pub(crate) mod pdf;
pub(crate) mod pptx;
mod structured;
mod text;
pub(crate) mod worker;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DocumentFormat {
    Docx,
    Pdf,
    Pptx,
    Text,
    Html,
}

impl DocumentFormat {
    /// The file extension this serialization is named by.
    pub(crate) const fn extension(self) -> &'static str {
        match self {
            Self::Docx => "docx",
            Self::Pdf => "pdf",
            Self::Pptx => "pptx",
            Self::Text => "txt",
            Self::Html => "html",
        }
    }

    fn render(self, source: &document::DocumentSource) -> Result<Vec<u8>, String> {
        match self {
            Self::Docx => docx::render(source),
            Self::Pdf => pdf::render(source),
            Self::Pptx => pptx::render(source),
            Self::Text => render_text(source),
            Self::Html => html::render(source, &source.title),
        }
    }
}

pub(crate) fn render_text(source: &document::DocumentSource) -> Result<Vec<u8>, String> {
    Ok(text::render(&markdown::parse(source)?).into_bytes())
}

#[cfg(test)]
mod tests;

pub(crate) mod document;
pub(crate) mod docx;
mod links;
mod markdown;
pub(crate) mod pdf;
pub(crate) mod pptx;
mod text;
pub(crate) mod worker;

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DocumentFormat {
    Docx,
    Pdf,
    Pptx,
    Text,
}

impl DocumentFormat {
    fn render(self, source: &document::DocumentSource) -> Result<Vec<u8>, String> {
        match self {
            Self::Docx => docx::render(source),
            Self::Pdf => pdf::render(source),
            Self::Pptx => pptx::render(source),
            Self::Text => render_text(source),
        }
    }
}

pub(crate) fn render_text(source: &document::DocumentSource) -> Result<Vec<u8>, String> {
    Ok(text::render(&markdown::parse(source)?).into_bytes())
}

#[cfg(test)]
mod tests;

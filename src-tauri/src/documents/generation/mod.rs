pub(crate) mod document;
pub(crate) mod docx;
mod links;
mod markdown;
pub(crate) mod pdf;
mod text;

pub(crate) fn render_text(source: &document::DocumentSource) -> Result<Vec<u8>, String> {
    Ok(text::render(&markdown::parse(source)?).into_bytes())
}

#[cfg(test)]
mod tests;

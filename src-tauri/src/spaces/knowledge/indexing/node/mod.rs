pub(super) mod document;
mod markdown;
mod note;
mod visual;
mod website;

use serde_json::Value;

use self::{
    markdown::markdown_chunks, note::note_chunks, visual::visual_chunks, website::website_chunks,
};

use super::KnowledgeChunkSeed;

pub(crate) fn derived_chunks(payload: &Value) -> Result<Vec<KnowledgeChunkSeed>, String> {
    let kind = payload.get("kind").and_then(Value::as_str);
    if kind == Some("file") {
        return document::chunks(payload);
    }
    if matches!(kind, Some("image" | "video")) {
        return visual_chunks(payload);
    }
    if !matches!(kind, Some("note" | "markdown" | "website")) {
        return Ok(Vec::new());
    }
    let revision = payload
        .get("knowledgeRevision")
        .and_then(Value::as_str)
        .ok_or_else(|| "Semantic Space node is missing knowledgeRevision".to_string())?;
    match kind {
        Some("note") => note_chunks(payload, revision),
        Some("markdown") => Ok(markdown_chunks(
            payload.get("text").and_then(Value::as_str).unwrap_or(""),
            revision,
        )),
        Some("website") => website_chunks(payload, revision),
        _ => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests;

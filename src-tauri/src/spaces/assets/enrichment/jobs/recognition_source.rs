use std::collections::HashSet;

use serde_json::{json, Value};

/// One picture a node asks to have read, and where its text is cited from.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(super) struct RecognitionSource {
    pub picture: Picture,
    pub kind: &'static str,
    pub location: Value,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(super) enum Picture {
    /// The picture the node shows, resolved the way vision resolves it: an image with its edits
    /// rendered, a video by its cover.
    Shown,
    /// A picture parsing extracted from a document. Its text belongs to it, not to the document.
    Resource { id: String, path: String },
}

/// The pictures of a node that hold text no parser could reach. The decision is made once, here, so
/// the job that is queued and the job that runs cannot disagree about what a node owes.
pub(super) fn sources(payload: &Value) -> Vec<RecognitionSource> {
    match payload.get("kind").and_then(Value::as_str) {
        // A depth field is a picture of distance: it shares its node kind with photographs and
        // nothing else, so reading it could only produce noise.
        Some("image") if payload.get("imageDepth").is_none() => vec![shown("image_description")],
        Some("video") => vec![shown("video_cover")],
        Some("file") => document_sources(payload),
        _ => Vec::new(),
    }
}

fn shown(kind: &'static str) -> RecognitionSource {
    RecognitionSource {
        picture: Picture::Shown,
        kind,
        location: json!({ "kind": "node" }),
    }
}

/// Renders are content addressed, so a picture repeated through a document is read once and cited
/// from its first appearance: a letterhead must not outweigh the pages it decorates.
fn document_sources(payload: &Value) -> Vec<RecognitionSource> {
    let Some(coverage) = payload.get("documentCoverage") else {
        return Vec::new();
    };
    if coverage.get("sourceRevision") != payload.get("sourceRevision") {
        return Vec::new();
    }
    let resources = entries(coverage, "resources");
    let mut seen = HashSet::new();
    entries(coverage, "occurrences")
        .iter()
        .filter_map(|occurrence| {
            let id = occurrence.get("resourceId")?.as_str()?;
            if !seen.insert(id) {
                return None;
            }
            let resource = resources
                .iter()
                .find(|resource| resource.get("id").and_then(Value::as_str) == Some(id))?;
            let location = occurrence.get("location")?.clone();
            let pdf = location.get("kind").and_then(Value::as_str) == Some("pdf");
            Some(RecognitionSource {
                picture: Picture::Resource {
                    id: id.to_owned(),
                    path: resource.get("path")?.as_str()?.to_owned(),
                },
                kind: if pdf { "page" } else { "image_description" },
                location,
            })
        })
        .collect()
}

fn entries<'a>(coverage: &'a Value, field: &str) -> &'a [Value] {
    coverage
        .get(field)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

#[cfg(test)]
#[path = "tests/recognition_source.rs"]
mod tests;

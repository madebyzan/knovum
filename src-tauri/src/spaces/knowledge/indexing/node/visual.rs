use serde::Deserialize;
use serde_json::{json, Value};

use crate::spaces::knowledge::indexing::{self, KnowledgeChunkSeed};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VisualKnowledge {
    source_revision: String,
    description: String,
    detected_text: Option<String>,
    tags: Vec<String>,
}

pub(super) fn visual_chunks(data: &Value) -> Result<Vec<KnowledgeChunkSeed>, String> {
    let Some(value) = data.get("visualKnowledge") else {
        return Ok(Vec::new());
    };
    let knowledge: VisualKnowledge = serde_json::from_value(value.clone())
        .map_err(|error| format!("Validated visualKnowledge could not be decoded: {error}"))?;
    let node_revision = data
        .get("knowledgeRevision")
        .and_then(Value::as_str)
        .ok_or_else(|| "Visual knowledge chunks require node knowledgeRevision".to_owned())?;
    let text = visual_text(&knowledge);
    let kind = match data.get("kind").and_then(Value::as_str) {
        Some("image") => "image_description",
        Some("video") => "video_cover",
        _ => return Ok(Vec::new()),
    };
    Ok(indexing::chunks(
        kind,
        &text,
        node_revision,
        json!({ "kind": "node" }),
        json!({
            "role": "visual",
            "assetSourceRevision": knowledge.source_revision,
            "tags": knowledge.tags.join(", ")
        }),
    ))
}

fn visual_text(knowledge: &VisualKnowledge) -> String {
    [
        knowledge.description.trim().to_owned(),
        knowledge
            .detected_text
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| format!("Visible text:\n{value}"))
            .unwrap_or_default(),
        if !knowledge.tags.is_empty() {
            format!("Tags: {}", knowledge.tags.join(", "))
        } else {
            String::new()
        },
    ]
    .into_iter()
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join("\n\n")
}

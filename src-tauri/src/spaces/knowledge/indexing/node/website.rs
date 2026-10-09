use serde_json::Value;

use crate::spaces::knowledge::indexing::{self, KnowledgeChunkSeed};

pub(super) fn website_chunks(
    payload: &Value,
    revision: &str,
) -> Result<Vec<KnowledgeChunkSeed>, String> {
    let mut seeds = Vec::new();
    let sections = payload
        .get("websiteSections")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for (index, section) in sections.iter().enumerate() {
        let text = section
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if text.is_empty() {
            continue;
        }
        let heading = section
            .get("heading")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty());
        for (part, passage) in indexing::split_passages_with_offsets(text)
            .into_iter()
            .enumerate()
        {
            let mut location = serde_json::json!({
                "kind":"website", "startOffset":passage.start_offset, "endOffset":passage.end_offset,
            });
            let mut metadata = serde_json::json!({"index":index,"part":part});
            if let Some(heading) = heading {
                location["section"] = Value::String(heading.to_owned());
                metadata["heading"] = Value::String(heading.to_owned());
            }
            seeds.push(KnowledgeChunkSeed {
                kind: if heading.is_some() { "heading" } else { "text" },
                locator: indexing::locator(revision, location, &passage.text),
                metadata,
                text: passage.text,
            });
        }
    }
    if !seeds.is_empty() {
        return Ok(seeds);
    }
    let preview = payload
        .get("contentPreview")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    for (part, passage) in indexing::split_passages_with_offsets(preview)
        .into_iter()
        .enumerate()
    {
        seeds.push(KnowledgeChunkSeed {
            kind: "text",
            locator: indexing::locator(revision, serde_json::json!({
                "kind":"website", "startOffset":passage.start_offset, "endOffset":passage.end_offset,
            }), &passage.text),
            metadata: serde_json::json!({"part":part}), text: passage.text,
        });
    }
    Ok(seeds)
}

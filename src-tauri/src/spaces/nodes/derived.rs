use serde_json::Value;

use crate::spaces::knowledge::indexing::{media as media_knowledge, node as node_knowledge};

pub(super) fn generated_chunks(
    data: &Value,
) -> Result<Vec<crate::spaces::knowledge::indexing::KnowledgeChunkInput>, String> {
    convert_chunks(node_knowledge::derived_chunks(data)?)
}

pub(super) fn media_chunks(
    data: &Value,
) -> Result<Vec<crate::spaces::knowledge::indexing::KnowledgeChunkInput>, String> {
    convert_chunks(media_knowledge::chunks(data)?)
}

fn convert_chunks(
    chunks: Vec<crate::spaces::knowledge::indexing::KnowledgeChunkSeed>,
) -> Result<Vec<crate::spaces::knowledge::indexing::KnowledgeChunkInput>, String> {
    chunks
        .into_iter()
        .map(|chunk| {
            Ok(crate::spaces::knowledge::indexing::KnowledgeChunkInput {
                kind: chunk.kind.to_owned(),
                text: chunk.text,
                locator: chunk.locator,
                metadata: (!chunk.metadata.is_null()).then_some(chunk.metadata),
            })
        })
        .collect()
}

pub(super) fn changed_top_level_fields(before: &Value, after: &Value) -> Vec<String> {
    let before = before.as_object();
    let after = after.as_object();
    let mut keys = std::collections::BTreeSet::new();
    if let Some(object) = before {
        keys.extend(object.keys().cloned());
    }
    if let Some(object) = after {
        keys.extend(object.keys().cloned());
    }
    keys.into_iter()
        .filter(|key| {
            before.and_then(|value| value.get(key)) != after.and_then(|value| value.get(key))
        })
        .collect()
}

pub(super) fn field_changed(before: &Value, after: &Value, field: &str) -> bool {
    before.get(field) != after.get(field)
}

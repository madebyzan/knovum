use serde_json::Value;

use crate::spaces::knowledge::indexing::{media as media_knowledge, node as node_knowledge};

pub(super) fn generated_chunks(
    data: &Value,
) -> Result<Vec<crate::spaces::knowledge::indexing::KnowledgeChunkInput>, String> {
    Ok(node_knowledge::derived_chunks(data)?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub(super) fn media_chunks(
    data: &Value,
) -> Result<Vec<crate::spaces::knowledge::indexing::KnowledgeChunkInput>, String> {
    Ok(media_knowledge::chunks(data)?
        .into_iter()
        .map(Into::into)
        .collect())
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

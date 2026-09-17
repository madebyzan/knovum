pub(crate) mod media;
pub(crate) mod node;
pub(crate) mod revision;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_KNOWLEDGE_PASSAGE_BYTES: usize = 48 * 1024;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeChunkSeed {
    pub kind: &'static str,
    pub text: String,
    pub locator: Value,
    #[serde(skip_serializing_if = "Value::is_null")]
    pub metadata: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeChunkInput {
    pub kind: String,
    pub text: String,
    pub locator: Value,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct KnowledgePassage {
    pub text: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

pub fn locator(source_revision: &str, location: Value, exact: &str) -> Value {
    let quote = if exact.trim().is_empty() {
        Value::Null
    } else {
        serde_json::json!({ "exact": exact })
    };
    let mut value = serde_json::json!({
        "sourceRevision": source_revision,
        "location": location,
    });
    if !quote.is_null() {
        value["quote"] = quote;
    }
    value
}

pub fn chunks(
    kind: &'static str,
    text: &str,
    source_revision: &str,
    location: Value,
    metadata: Value,
) -> Vec<KnowledgeChunkSeed> {
    split_passages_with_offsets(text)
        .into_iter()
        .map(|part| {
            let passage_location =
                location_for_passage(&location, part.start_offset, part.end_offset);
            KnowledgeChunkSeed {
                kind,
                locator: locator(source_revision, passage_location, &part.text),
                metadata: metadata.clone(),
                text: part.text,
            }
        })
        .collect()
}

mod passages;
pub use passages::{location_for_passage, split_passages_with_offsets};

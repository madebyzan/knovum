use super::validation::{validate_kind, validate_metadata};
use crate::spaces::knowledge::indexing::MAX_KNOWLEDGE_PASSAGE_BYTES;
use serde_json::json;

#[test]
fn metadata_rejects_nested_values() {
    assert!(
        validate_metadata(&json!({ "page": 2, "label": "ok", "flag": true, "empty": null }))
            .is_ok()
    );
    assert!(validate_metadata(&json!({ "nested": { "value": 1 } })).is_err());
    assert!(validate_metadata(&json!({ "list": [1, 2] })).is_err());
}

#[test]
fn kind_rejects_unknown_chunk_types() {
    assert!(validate_kind("text").is_ok());
    assert!(validate_kind("unknown").is_err());
    const { assert!(MAX_KNOWLEDGE_PASSAGE_BYTES < 64 * 1024) };
}

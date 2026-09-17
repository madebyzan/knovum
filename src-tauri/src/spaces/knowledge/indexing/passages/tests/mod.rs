use super::{location_for_passage, split_passages_with_offsets};
use crate::spaces::knowledge::indexing::MAX_KNOWLEDGE_PASSAGE_BYTES;
use serde_json::json;

fn split_passages(text: &str) -> Vec<String> {
    split_passages_with_offsets(text)
        .into_iter()
        .map(|part| part.text)
        .collect()
}

#[test]
fn passage_split_respects_embedding_byte_limit() {
    let text = "😀 paragraph ".repeat(10_000);
    let parts = split_passages(&text);
    assert!(parts.len() > 1);
    assert!(parts
        .iter()
        .all(|part| !part.is_empty() && part.len() <= MAX_KNOWLEDGE_PASSAGE_BYTES));
    assert_eq!(
        parts.join(" ").replace("  ", " ").trim(),
        text.trim().replace("  ", " ")
    );
}

#[test]
fn passage_offsets_are_utf16_and_adjust_existing_locations() {
    let text = format!("  {} tail", "😀".repeat(30_000));
    let parts = split_passages_with_offsets(&text);
    assert!(parts.len() > 1);
    assert_eq!(parts[0].start_offset, 2);
    let location = location_for_passage(
        &json!({"kind":"markdown","startOffset":100,"endOffset":999}),
        parts[0].start_offset,
        parts[0].end_offset,
    );
    assert_eq!(location["startOffset"], 102);
    assert!(location["endOffset"].as_u64().unwrap() > 102);
}

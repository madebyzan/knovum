use serde_json::Value;

use crate::spaces::knowledge::indexing::{self, KnowledgeChunkSeed};

pub(super) fn note_chunks(data: &Value, revision: &str) -> Result<Vec<KnowledgeChunkSeed>, String> {
    let root = data
        .get("tiptapContent")
        .and_then(Value::as_object)
        .ok_or_else(|| "Note tiptapContent must be an object".to_string())?;
    let blocks = root
        .get("content")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut chunks = Vec::new();
    let mut headings: Vec<String> = Vec::new();
    for (index, block) in blocks.iter().enumerate() {
        let text = node_text(block).trim().to_owned();
        if text.is_empty() {
            continue;
        }
        let level = heading_level(block);
        if level > 0 {
            headings.truncate(level.saturating_sub(1));
            while headings.len() < level.saturating_sub(1) {
                headings.push(String::new());
            }
            headings.push(text.clone());
        }
        let stored = block
            .get("attrs")
            .and_then(Value::as_object)
            .and_then(|attrs| attrs.get("id"))
            .and_then(Value::as_str)
            .filter(|value| value.encode_utf16().count() <= 16_000);
        let block_id = stored
            .map(str::to_owned)
            .unwrap_or_else(|| format!("block-{index}"));
        let heading = headings
            .iter()
            .filter(|value| !value.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(" / ");
        let heading = crate::documents::extraction::text::bounded_utf16_text(&heading, 512);
        for (part, passage) in indexing::split_passages_with_offsets(&text)
            .into_iter()
            .enumerate()
        {
            let location = serde_json::json!({
                "kind": "note", "blockId": block_id,
                "startOffset": passage.start_offset, "endOffset": passage.end_offset,
            });
            let mut metadata = serde_json::json!({"blockId": block_id, "part": part});
            if !heading.is_empty() {
                metadata["heading"] = Value::String(heading.clone());
            }
            chunks.push(KnowledgeChunkSeed {
                kind: if level > 0 { "heading" } else { "text" },
                locator: indexing::locator(revision, location, &passage.text),
                metadata,
                text: passage.text,
            });
        }
    }
    Ok(chunks)
}

fn node_text(node: &Value) -> String {
    if let Some(text) = node.get("text").and_then(Value::as_str) {
        return text.to_owned();
    }
    let separator = if node.get("type").and_then(Value::as_str) == Some("hardBreak") {
        "\n"
    } else {
        ""
    };
    node.get("content")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(node_text)
                .collect::<Vec<_>>()
                .join(separator)
        })
        .unwrap_or_default()
}

fn heading_level(block: &Value) -> usize {
    if block.get("type").and_then(Value::as_str) != Some("heading") {
        return 0;
    }
    block
        .get("attrs")
        .and_then(Value::as_object)
        .and_then(|attrs| attrs.get("level"))
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .clamp(1, 6) as usize
}

use serde_json::json;

use crate::spaces::{assets::PreparedAsset, knowledge::indexing};

pub(super) fn attach(asset: &mut PreparedAsset, revision: &str) {
    let Some(text) = asset
        .preview_text
        .as_ref()
        .filter(|text| !text.trim().is_empty())
    else {
        return;
    };
    if asset.kind == "code" {
        asset.knowledge_chunks = Some(code_chunks(text, revision));
        return;
    }
    let seeds = indexing::chunks("text", text, revision, json!({ "kind": "node" }), json!({}));
    asset.knowledge_chunks = (!seeds.is_empty()).then_some(seeds);
}

fn code_chunks(text: &str, revision: &str) -> Vec<indexing::KnowledgeChunkSeed> {
    const WINDOW: usize = 80;
    const OVERLAP: usize = 10;
    let lines = text.lines().collect::<Vec<_>>();
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < lines.len() {
        let end = (start + WINDOW).min(lines.len());
        chunks.extend(indexing::chunks(
            "code",
            &lines[start..end].join("\n"),
            revision,
            json!({ "kind": "code", "lineStart": start + 1, "lineEnd": end }),
            json!({ "lineStart": start + 1, "lineEnd": end }),
        ));
        if end == lines.len() {
            break;
        }
        start = end.saturating_sub(OVERLAP);
    }
    chunks
}

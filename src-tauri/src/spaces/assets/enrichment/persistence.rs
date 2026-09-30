use serde_json::Value;
use tauri::AppHandle;

use super::super::PreparedAsset;
use crate::spaces::knowledge::indexing::KnowledgeChunkInput;

pub(in crate::spaces) struct PublicationGuard<'a> {
    pub source_revision: Option<&'a str>,
    pub job_id: Option<&'a str>,
}

pub(in crate::spaces::assets) async fn persist_processing_state(
    app: &AppHandle,
    space_id: &str,
    node_id: &str,
    data: &mut Value,
    state: &str,
    chunks: Option<Vec<KnowledgeChunkInput>>,
    guard: Option<&PublicationGuard<'_>>,
) -> Result<(), String> {
    let object = data
        .as_object_mut()
        .ok_or_else(|| "Space node data must be an object".to_owned())?;
    if state == "done" {
        object.remove("processingState");
    } else {
        object.insert("processingState".into(), Value::String(state.to_owned()));
    }
    let revision = data
        .get("sourceRevision")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let expected = guard.map_or(revision.as_deref(), |guard| guard.source_revision);
    let fields = [
        "previewPath",
        "previewText",
        "extractedText",
        "intrinsicWidth",
        "intrinsicHeight",
        "documentCoverage",
        "processingState",
        "sourceRevision",
        "previewLanguage",
        "size",
        "modifiedAt",
        "entries",
    ]
    .into_iter()
    .map(|key| {
        (
            key.to_owned(),
            data.get(key).cloned().unwrap_or(Value::Null),
        )
    })
    .collect();
    *data = super::commit::publish(
        app,
        space_id,
        node_id,
        super::commit::PublicationGuard {
            source_revision: expected,
            job_id: guard.and_then(|guard| guard.job_id),
            media_knowledge: None,
            document_coverage: None,
            cancellation: None,
        },
        fields,
        chunks,
    )
    .await?;
    Ok(())
}

pub(super) fn take_chunks(asset: &mut PreparedAsset) -> Vec<KnowledgeChunkInput> {
    asset
        .knowledge_chunks
        .take()
        .unwrap_or_default()
        .into_iter()
        .map(|chunk| KnowledgeChunkInput {
            kind: chunk.kind.to_owned(),
            text: chunk.text,
            locator: chunk.locator,
            metadata: (!chunk.metadata.is_null()).then_some(chunk.metadata),
        })
        .collect()
}

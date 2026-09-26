use std::path::Path;

use serde_json::Value;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::{
    apply_asset_fields, ensure_active, node_payload::optional_string, persistence::take_chunks,
};
use crate::spaces::{assets::PreparedAsset, knowledge::indexing::KnowledgeChunkInput, previews};

pub(super) struct ContentOutput {
    pub data: Value,
    pub chunks: Vec<KnowledgeChunkInput>,
}

pub(super) struct VisualOutput {
    pub asset: PreparedAsset,
    pub data: Value,
}

pub(super) fn prepare_visual(
    app: &AppHandle,
    source: &Path,
    file_name: &str,
    revision: &str,
    kind: &str,
    current: &Value,
    cancellation: &CancellationToken,
) -> Result<VisualOutput, String> {
    let mut asset = asset_from_data(source, file_name, kind, revision, current)?;
    ensure_active(cancellation)?;
    previews::enrich_visual(app, source, &mut asset, cancellation)?;
    ensure_active(cancellation)?;
    let mut data = current.clone();
    apply_asset_fields(&mut data, &asset)?;
    Ok(VisualOutput { asset, data })
}

pub(super) fn enrich_content(
    app: &AppHandle,
    source: &Path,
    revision: &str,
    mut asset: PreparedAsset,
    mut data: Value,
    cancellation: &CancellationToken,
) -> Result<ContentOutput, String> {
    let result = super::content::extract(app, source, revision, &mut asset, cancellation);
    ensure_active(cancellation)?;
    result?;
    let chunks = take_chunks(&mut asset);
    apply_asset_fields(&mut data, &asset)?;
    Ok(ContentOutput { data, chunks })
}

fn asset_from_data(
    source: &Path,
    file_name: &str,
    kind: &str,
    revision: &str,
    data: &Value,
) -> Result<PreparedAsset, String> {
    let kind = match kind {
        "image" => "image",
        "video" => "video",
        "audio" => "audio",
        "file" => "file",
        "code" => "code",
        other => return Err(format!("Asset kind {other} cannot be enriched")),
    };
    Ok(PreparedAsset {
        name: file_name.to_owned(),
        path: Some(source.to_string_lossy().into_owned()),
        kind,
        text: None,
        preview_text: optional_string(data, "previewText"),
        preview_language: optional_string(data, "previewLanguage"),
        preview_path: optional_string(data, "previewPath"),
        extracted_text: optional_string(data, "extractedText"),
        size: data.get("size").and_then(Value::as_u64),
        modified_at: data.get("modifiedAt").and_then(Value::as_u64),
        source_revision: Some(revision.to_owned()),
        processing_state: Some("processing"),
        intrinsic_width: optional_u32(data, "intrinsicWidth"),
        intrinsic_height: optional_u32(data, "intrinsicHeight"),
        knowledge_chunks: None,
        document_coverage: data.get("documentCoverage").cloned(),
        entries: data
            .get("entries")
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
    })
}

fn optional_u32(data: &Value, field: &str) -> Option<u32> {
    data.get(field)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

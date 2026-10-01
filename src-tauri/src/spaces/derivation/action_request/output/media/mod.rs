use serde_json::{json, Map, Value};

use super::super::inputs::{ResolvedInputs, ResolvedNode};

pub(super) fn build_subtitled_video_content(
    app: &tauri::AppHandle,
    title: &str,
) -> Result<Value, String> {
    Ok(
        json!({ "kind": "video", "title": title, "fileName": format!("{title}.mp4"),
        "assetPath": super::super::super::output_paths::reserve(app, "subtitled", "mp4")?,
        "processingState": "processing" }),
    )
}

pub(super) fn subtitled_video_request(
    inputs: &ResolvedInputs,
    request: &mut Map<String, Value>,
) -> Result<(), String> {
    let video = inputs
        .nodes
        .first()
        .ok_or("action-input-invalid: video is required")?;
    if inputs.nodes.len() != 1 || video.kind != "video" {
        return Err("action-input-invalid: adding subtitles requires one video source".into());
    }
    *request = Map::from_iter([
        ("sourceNodeId".into(), video.id.clone().into()),
        ("sourceRevision".into(), asset_revision(video)?.into()),
    ]);
    request.insert("knowledgeRevision".into(), video.revision.clone().into());
    Ok(())
}

pub(super) fn document_request(
    inputs: &ResolvedInputs,
    request: &mut Map<String, Value>,
) -> Result<(), String> {
    let document: &ResolvedNode = inputs
        .nodes
        .first()
        .ok_or("action-input-invalid: document is required")?;
    if inputs.nodes.len() != 1 || document.kind != "markdown" {
        return Err("action-input-invalid: document export requires one Markdown source".into());
    }
    request.insert(
        "document".into(),
        json!({"nodeId":document.id,"data":document.data}),
    );
    Ok(())
}

fn asset_revision(video: &ResolvedNode) -> Result<&str, String> {
    video
        .data
        .get("sourceRevision")
        .and_then(Value::as_str)
        .ok_or_else(|| "Video asset revision is unavailable".into())
}

#[cfg(test)]
mod tests;

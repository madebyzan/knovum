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
    action_id: &str,
    request: &mut Map<String, Value>,
) -> Result<(), String> {
    let document: &ResolvedNode = inputs
        .nodes
        .first()
        .ok_or("action-input-invalid: document is required")?;
    if inputs.nodes.len() != 1 {
        return Err("action-input-invalid: document conversion requires one source".into());
    }
    match document.kind.as_str() {
        "markdown" if document.data.get("derivationState").is_none() => {}
        "file" if document.data.get("processingState").is_none() => {
            let path = document
                .data
                .get("assetPath")
                .and_then(Value::as_str)
                .ok_or("action-input-invalid: document source path is required")?;
            let path = std::path::Path::new(path);
            if !crate::documents::generation::input::is_supported_source_format(path)
                || document
                    .data
                    .get("sourceRevision")
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty)
            {
                return Err(
                    "action-input-invalid: unsupported or unpublished document source".into(),
                );
            }
            let target = super::super::super::document_formats::require(action_id)?;
            if path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case(target.extension))
            {
                return Err(
                    "action-input-invalid: document already has the requested format".into(),
                );
            }
            if target.extension == "pptx"
                && !path.extension().is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("pdf") || extension.eq_ignore_ascii_case("ppt")
                })
            {
                return Err("action-input-invalid: slide conversion supports Markdown, PDF or legacy PowerPoint".into());
            }
        }
        _ => return Err("action-input-invalid: document source is unavailable".into()),
    }
    request.insert(
        "document".into(),
        json!({"nodeId":document.id,"kind":document.kind,"data":document.data}),
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

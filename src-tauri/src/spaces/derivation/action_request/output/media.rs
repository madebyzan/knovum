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
mod tests {
    use super::*;
    use crate::spaces::derivation::subtitled_video::RenderSubtitledVideoRequest;

    #[test]
    fn automatic_action_projects_one_video_with_distinct_asset_and_knowledge_revisions() {
        let inputs = ResolvedInputs {
            items: vec![],
            nodes: vec![ResolvedNode {
                id: "video".into(),
                kind: "video".into(),
                revision: "knowledge:1".into(),
                data: json!({ "sourceRevision": "asset:1" }),
            }],
        };
        let mut request =
            Map::from_iter([("inputs".into(), json!([])), ("primary".into(), json!({}))]);
        subtitled_video_request(&inputs, &mut request).unwrap();
        request.extend(Map::from_iter([
            ("id".into(), json!("job")),
            ("spaceId".into(), json!("space")),
            ("outputNodeId".into(), json!("output")),
        ]));
        let decoded: RenderSubtitledVideoRequest =
            serde_json::from_value(Value::Object(request)).unwrap();
        assert_eq!(decoded.source_revision, "asset:1");
        assert_eq!(decoded.knowledge_revision, "knowledge:1");
        let action = crate::spaces::actions::require("add-video-subtitles").unwrap();
        assert!(action.execution_capability.is_none());
        assert!(action.routing_purpose.is_none());
        assert_eq!(action.input_roles.len(), 1);
    }
}

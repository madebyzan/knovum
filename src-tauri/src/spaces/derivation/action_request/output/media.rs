use serde_json::{json, Map, Value};

use super::super::inputs::{ResolvedInputs, ResolvedNode};
use crate::spaces::knowledge::indexing::revision;

pub(super) fn subtitle_data(inputs: &ResolvedInputs) -> Result<Value, String> {
    let video = inputs
        .nodes
        .first()
        .ok_or("action-input-invalid: video is required")?;
    let transcript = video
        .data
        .pointer("/mediaKnowledge/transcript")
        .ok_or("action-input-invalid: analyze the video before extracting subtitles")?;
    let mut data = json!({
        "kind":"subtitle","title":"Extracted Subtitles","sourceVideoNodeId":video.id,
        "sourceVideoRevision":video.revision,"language":transcript.get("language"),
        "durationMs":transcript.get("durationMs"),"cues":transcript.get("subtitleCues")
    });
    revision::apply(&mut data)?;
    Ok(data)
}

pub(super) fn subtitled_video_data(inputs: &ResolvedInputs, title: &str) -> Result<Value, String> {
    let mut data = inputs
        .nodes
        .first()
        .ok_or("action-input-invalid: video is required")?
        .data
        .clone();
    let object = data
        .as_object_mut()
        .ok_or("action-input-invalid: video data is invalid")?;
    object.insert("title".into(), title.into());
    object.insert("fileName".into(), format!("{title}.mp4").into());
    object.insert("processingState".into(), "processing".into());
    Ok(data)
}

pub(super) fn subtitle_request(
    inputs: &ResolvedInputs,
    request: &mut Map<String, Value>,
) -> Result<(), String> {
    let video = inputs
        .nodes
        .first()
        .ok_or("action-input-invalid: video is required")?;
    request.insert("sourceNodeId".into(), video.id.clone().into());
    request.insert("sourceRevision".into(), video.revision.clone().into());
    Ok(())
}

pub(super) fn subtitled_video_request(
    inputs: &ResolvedInputs,
    request: &mut Map<String, Value>,
) -> Result<(), String> {
    let video = inputs
        .nodes
        .first()
        .ok_or("action-input-invalid: video is required")?;
    let subtitle = inputs
        .nodes
        .iter()
        .find(|node| node.kind == "subtitle")
        .ok_or("action-input-invalid: subtitles are required")?;
    request.insert("sourceNodeId".into(), video.id.clone().into());
    request.insert("sourceRevision".into(), video.revision.clone().into());
    request.insert("subtitleNodeId".into(), subtitle.id.clone().into());
    request.insert("subtitleRevision".into(), subtitle.revision.clone().into());
    Ok(())
}

pub(super) fn presentation_request(
    inputs: &ResolvedInputs,
    request: &mut Map<String, Value>,
) -> Result<(), String> {
    let document: &ResolvedNode = inputs
        .nodes
        .first()
        .ok_or("action-input-invalid: document is required")?;
    request.insert(
        "document".into(),
        json!({"nodeId":document.id,"data":document.data}),
    );
    Ok(())
}

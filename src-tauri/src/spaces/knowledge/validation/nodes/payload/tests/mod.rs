use super::validate_payload_shape;
use serde_json::json;

#[test]
fn directory_and_archive_node_kinds_are_not_space_contracts() {
    for kind in ["folder", "archive"] {
        let data = json!({"kind": kind, "title": "Project", "fileName": "Project", "assetPath": "/assets/project"});
        assert_eq!(
            validate_payload_shape(kind, &data).unwrap_err(),
            "Unsupported Space node kind"
        );
    }
}

fn video() -> serde_json::Value {
    json!({
        "kind": "video", "title": "Video", "assetPath": "/video.mp4", "fileName": "video.mp4",
        "sourceRevision": "revision",
        "videoEditProject": serde_json::from_str::<serde_json::Value>(r#"{
 "version":1,"sourceRevision":"revision",
 "assets":[{"nodeId":"source","sourceRevision":"revision","kind":"video","durationMs":8000,"width":1080,"height":1920}],
 "output":{"width":1080,"height":1920,"frameRate":30},
 "videoTrack":{"clips":[
 {"id":"d51a4384-70ab-4b44-90cc-3bfcf0f5341f","assetId":"source","startMs":1000,"endMs":3000,"volume":1,"fadeInMs":0,"fadeOutMs":0,"transition":null,"transform":{"rotation":90,"flipHorizontal":true,"flipVertical":false,"fit":"contain","zoom":1}},
 {"id":"d51a4384-70ab-4b44-90cc-3bfcf0f5342f","assetId":"source","startMs":5000,"endMs":8000,"volume":0.5,"fadeInMs":0,"fadeOutMs":0,"transition":null,"transform":{"rotation":0,"flipHorizontal":false,"flipVertical":false,"fit":"cover","zoom":1}}]},
 "audioTrack":{"clips":[]},"textTrack":{"items":[]}
}"#).unwrap()
    })
}

#[test]
fn editor_payload_fields_reach_the_node_save_boundary() {
    validate_payload_shape("video", &video()).unwrap();
    let markdown = json!({
        "kind": "markdown", "title": "Document", "text": "Draft",
        "tiptapContent": {"type": "doc", "content": [{"type": "paragraph", "content": [{"type": "text", "text": "Draft"}]}]}
    });
    validate_payload_shape("markdown", &markdown).unwrap();
}

#[test]
fn saved_video_projects_require_a_current_source_and_valid_project() {
    let mut data = video();
    data["videoEditProject"]["sourceRevision"] = json!("stale");
    assert!(validate_payload_shape("video", &data).is_err());
    data = video();
    data["videoEditProject"]["version"] = json!(2);
    assert!(validate_payload_shape("video", &data).is_err());
    data = video();
    data["videoEditProject"]["videoTrack"]["clips"][0]["endMs"] = json!(0);
    assert!(validate_payload_shape("video", &data).is_err());
    data = video();
    data["videoEditProject"]["unexpected"] = json!(true);
    assert!(validate_payload_shape("video", &data).is_err());
}

#[test]
fn markdown_rich_text_uses_the_document_boundary_validation() {
    let mut data = json!({"kind": "markdown", "title": "Document", "text": "Draft", "tiptapContent": {"type": "paragraph"}});
    assert!(validate_payload_shape("markdown", &data).is_err());
    data["tiptapContent"] = json!({"type": "doc", "unexpected": true});
    assert!(validate_payload_shape("markdown", &data).is_err());
    data["tiptapContent"] = json!({"type": "doc", "content": "invalid"});
    assert!(validate_payload_shape("markdown", &data).is_err());
}

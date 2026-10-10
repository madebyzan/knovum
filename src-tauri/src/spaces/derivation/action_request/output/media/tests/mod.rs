use super::*;
use crate::spaces::derivation::subtitled_video::RenderSubtitledVideoRequest;

#[test]
fn document_conversion_admits_one_published_pdf_and_rejects_binary_or_duplicate_formats() {
    let mut inputs = ResolvedInputs {
        items: vec![],
        nodes: vec![ResolvedNode {
            id: "source".into(),
            kind: "file".into(),
            revision: "knowledge:1".into(),
            data: json!({"kind":"file", "title":"Report", "assetPath":"/owned/report.pdf", "sourceRevision":"sha256:current"}),
        }],
    };
    let mut request = Map::new();
    document_request(&inputs, "convert-document-to-docx", &mut request).unwrap();
    assert_eq!(request["document"]["nodeId"], "source");
    assert_eq!(request["document"]["kind"], "file");
    assert!(document_request(&inputs, "convert-document-to-pdf", &mut request).is_err());
    inputs.nodes[0].data["assetPath"] = json!("/owned/binary.bin");
    assert!(document_request(&inputs, "convert-document-to-docx", &mut request).is_err());
    inputs.nodes[0].data["assetPath"] = json!("/owned/report.docx");
    inputs.nodes[0].data["processingState"] = json!("processing");
    assert!(document_request(&inputs, "convert-document-to-pdf", &mut request).is_err());
}

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
    let mut request = Map::from_iter([("inputs".into(), json!([])), ("primary".into(), json!({}))]);
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

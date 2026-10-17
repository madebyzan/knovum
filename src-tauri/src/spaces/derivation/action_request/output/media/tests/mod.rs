use super::*;
use crate::spaces::derivation::action_request::inputs::PinnedRevision;
use crate::spaces::derivation::inputs::RevisionBasis;
use crate::spaces::derivation::subtitled_video::RenderSubtitledVideoRequest;

#[test]
fn automatic_action_projects_one_video_with_distinct_asset_and_knowledge_revisions() {
    let inputs = ResolvedInputs {
        items: vec![],
        nodes: vec![ResolvedNode {
            id: "video".into(),
            kind: "video".into(),
            revision: PinnedRevision {
                basis: RevisionBasis::Knowledge,
                value: "knowledge:1".into(),
            },
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

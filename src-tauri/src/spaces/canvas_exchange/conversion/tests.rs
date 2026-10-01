use super::*;
#[test]
fn preserves_group_background_styles_with_the_managed_resource() {
    let frame: NodeFrameV1 =
        serde_json::from_value(json!({"id":"section","x":0,"y":0,"width":400,"height":300}))
            .unwrap();
    for style in ["cover", "ratio", "repeat"] {
        let parsed = serde_json::from_value(json!(style)).unwrap();
        let appearance = section_appearance(
            &frame,
            Some("/managed/background.png".into()),
            Some(&parsed),
        )
        .unwrap();
        assert_eq!(appearance["background"], "/managed/background.png");
        assert_eq!(appearance["backgroundStyle"], style);
    }
}
#[test]
fn adopts_unambiguous_section_membership_and_rejects_group_edges() {
    let mut document: JsonCanvasDocumentV1 = serde_json::from_value(json!({"nodes":[{"id":"section","type":"group","x":0,"y":0,"width":400,"height":300},{"id":"note","type":"text","text":"Evidence","x":20,"y":40,"width":200,"height":120}]})).unwrap();
    assert_eq!(
        memberships(&document).unwrap().get("note").unwrap(),
        "section"
    );
    document.edges.push(
        serde_json::from_value(json!({"id":"edge","fromNode":"section","toNode":"note"})).unwrap(),
    );
    assert!(memberships(&document)
        .unwrap_err()
        .message
        .contains("visual groups"));
}
#[test]
fn maps_edges_to_semantic_relations_and_rejects_unknown_targets() {
    let edge =
        serde_json::from_value(json!({"id":"edge","fromNode":"a","toNode":"b","label":"supports"}))
            .unwrap();
    let identities = HashMap::from([("a".into(), "node-a".into()), ("b".into(), "node-b".into())]);
    let mapped = relation(&edge, &identities, "space").unwrap();
    assert_eq!(mapped.source_id, "node-a");
    assert_eq!(mapped.target_id, "node-b");
    assert_eq!(mapped.kind, "related");
    assert_eq!(mapped.rationale, "supports");
    assert!(relation(&edge, &HashMap::new(), "space").is_err());
}
#[test]
fn rejects_ambiguous_membership_and_undersized_sections() {
    let document = serde_json::from_value(json!({"nodes":[{"id":"left","type":"group","x":0,"y":0,"width":400,"height":300},{"id":"right","type":"group","x":0,"y":0,"width":400,"height":300},{"id":"note","type":"text","text":"Evidence","x":20,"y":40,"width":200,"height":120}]})).unwrap();
    assert!(memberships(&document)
        .unwrap_err()
        .message
        .contains("ambiguous"));
    let document = serde_json::from_value(
        json!({"nodes":[{"id":"small","type":"group","x":0,"y":0,"width":239,"height":160}]}),
    )
    .unwrap();
    assert!(memberships(&document)
        .unwrap_err()
        .message
        .contains("too small"));
}

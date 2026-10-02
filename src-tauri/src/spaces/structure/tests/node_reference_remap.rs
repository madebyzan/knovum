use std::collections::HashMap;

use serde_json::json;

use super::super::node_reference_remap;

#[test]
fn remaps_only_declared_object_references() {
    let mut data = json!({
        "kind": "object", "title": "old-node", "tags": [], "status": "draft",
        "properties": {"description": "old-node", "link": "old-node"}
    });
    let fields = json!([
        {"key": "description", "kind": "text"},
        {"key": "link", "kind": "node-ref"}
    ]);
    node_reference_remap::remap(
        &mut data,
        &HashMap::from([("old-node".into(), "new-node".into())]),
        &HashMap::new(),
        Some(&fields),
    )
    .unwrap();
    assert_eq!(data["title"], "old-node");
    assert_eq!(data["properties"]["description"], "old-node");
    assert_eq!(data["properties"]["link"], "new-node");
    assert!(data["knowledgeRevision"].as_str().is_some());
}

#[test]
fn remaps_portal_targets_without_changing_unrelated_text() {
    let mut data = json!({
        "kind": "portal", "title": "old-node", "targetNodeId": "old-node"
    });
    node_reference_remap::remap(
        &mut data,
        &HashMap::from([("old-node".into(), "new-node".into())]),
        &HashMap::from([("old-structure".into(), "new-structure".into())]),
        None,
    )
    .unwrap();
    assert_eq!(data["title"], "old-node");
    assert_eq!(data["targetNodeId"], "new-node");
    let mut structure_portal = json!({
        "kind": "portal", "title": "Container", "targetStructureNodeId": "old-structure"
    });
    node_reference_remap::remap(
        &mut structure_portal,
        &HashMap::new(),
        &HashMap::from([("old-structure".into(), "new-structure".into())]),
        None,
    )
    .unwrap();
    assert_eq!(structure_portal["targetStructureNodeId"], "new-structure");
}

#[test]
fn leaves_editor_content_with_id_like_text_unchanged() {
    let mut data = json!({
        "kind": "note", "title": "Note", "plainText": "old-node",
        "tiptapContent": {"type": "doc", "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": "old-node"}]}
        ]}
    });
    let before = data.clone();
    node_reference_remap::remap(
        &mut data,
        &HashMap::from([("old-node".into(), "new-node".into())]),
        &HashMap::new(),
        None,
    )
    .unwrap();
    assert_eq!(data, before);
}

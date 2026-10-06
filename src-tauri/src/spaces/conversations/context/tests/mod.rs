#[tokio::test]
async fn space_snapshot_remains_distinct_from_the_user_input() {
    let context = super::ResolvedContext {
        manifest: serde_json::json!([]),
        summaries: vec![],
        excerpts: vec![],
        images: vec![],
        agent_paths: vec![],
        space: crate::spaces::queries::knowledge_projection::ContentsPage {
            space_id: "current-space".into(),
            title: "Inbox".into(),
            total_nodes: 1,
            nodes: vec![
                crate::spaces::queries::knowledge_projection::contents::ContentNode {
                    node_id: "existing-image".into(),
                    kind: "image".into(),
                    title: "Girl".into(),
                    knowledge_revision: Some("node1:current".into()),
                    source_revision: Some("sha256:asset".into()),
                    indexed: false,
                    updated_at: 1,
                },
            ],
            next_offset: None,
        },
    };
    let message = context.initial_message(
        "input:test".into(),
        "What is in this Space?",
        Some("Created Girl earlier"),
    );
    let crate::agent_runtime::protocol::messages::AgentContentPart::Text { text } =
        &message.content[0]
    else {
        panic!("context text");
    };
    assert_eq!(context.space.space_id, "current-space");
    assert_eq!(context.space.nodes[0].node_id, "existing-image");
    assert!(!context.space.nodes[0].indexed);
    assert!(!text.contains("current-space"));
    assert!(text.contains("What is in this Space?"));
    assert!(text.contains("Created Girl earlier"));
}

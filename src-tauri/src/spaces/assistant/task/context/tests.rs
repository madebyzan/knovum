#[tokio::test]
async fn even_an_unattached_message_contains_space_identity_and_unindexed_canvas_resources() {
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
    let target = crate::ai::models::ExecutionTarget {
        provider: "openai".into(),
        model: "test".into(),
        base_url: "https://unused.invalid".into(),
        temperature: 0.0,
        secret_key: String::new(),
    };
    let message = context
        .initial_message(
            "What is in this Space?",
            Some("Created Girl earlier"),
            &target,
        )
        .await
        .unwrap();
    let crate::ai::contracts::ModelContentPart::Text { text } = &message.content[0] else {
        panic!("context text");
    };
    assert!(text.contains("current-space"));
    assert!(text.contains("existing-image"));
    assert!(text.contains("\"indexed\":false"));
    assert!(text.contains("What is in this Space?"));
    assert!(text.contains("Created Girl earlier"));
}

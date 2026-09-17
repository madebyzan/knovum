use crate::spaces::knowledge::grounding::prompt;

#[test]
fn citation_ids_follow_streamed_markdown_order() {
    assert_eq!(
        prompt::citation_indices("First [C2]. Second [C1][C2]."),
        vec![2, 1, 2]
    );
}

#[test]
fn conversation_prompt_stays_within_budget() {
    let history = (0..8)
        .map(|index| ("user".to_owned(), "😀".repeat(300_000 + index)))
        .collect::<Vec<_>>();
    let sources = vec![serde_json::json!({
        "nodeTitle": "Source",
        "text": "Grounded fact",
    })];
    let prompt = prompt::build("Question", &history, &sources).expect("prompt should fit");
    assert!(prompt.len() <= 48_000);
    assert!(!prompt.contains('\u{fffd}'));
}

#[test]
fn citations_include_explicit_index() {
    let sources = vec![
        serde_json::json!({
            "nodeId": "node-1",
            "chunkId": "chunk-1",
            "nodeTitle": "Title 1",
            "sourceRevision": "rev-1",
            "locator": { "sourceRevision": "rev-1", "location": { "kind": "node" } },
            "text": "Text 1",
        }),
        serde_json::json!({
            "nodeId": "node-2",
            "chunkId": "chunk-2",
            "nodeTitle": "Title 2",
            "sourceRevision": "rev-2",
            "locator": { "sourceRevision": "rev-2", "location": { "kind": "node" } },
            "text": "Text 2",
        }),
    ];
    let generated = prompt::citations("Claims from [C2] and [C1].", &sources, "space")
        .expect("citations to parse");
    let items = generated.as_array().expect("array of citations");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["index"], 2);
    assert_eq!(items[0]["nodeTitle"], "Title 2");
    assert_eq!(items[1]["index"], 1);
    assert_eq!(items[1]["nodeTitle"], "Title 1");
}

#[test]
fn citations_drop_indices_that_reference_no_source() {
    let sources = vec![serde_json::json!({
        "nodeId": "node-1",
        "chunkId": "chunk-1",
        "nodeTitle": "Title 1",
        "sourceRevision": "rev-1",
        "locator": { "sourceRevision": "rev-1", "location": { "kind": "node" } },
        "text": "Text 1",
    })];
    let generated = prompt::citations("Under [C0], valid [C1], beyond [C7].", &sources, "space")
        .expect("citations to parse");
    let items = generated.as_array().expect("array of citations");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["index"], 1);
    assert_eq!(items[0]["nodeTitle"], "Title 1");
}

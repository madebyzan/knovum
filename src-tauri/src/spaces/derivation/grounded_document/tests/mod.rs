use serde_json::json;

use super::{completion::apply_document, finalize_generated_document, prompt::request};

#[test]
fn a_document_with_sources_is_held_to_them_and_cites_them() {
    let input = request(
        "Brief",
        &[json!({
            "nodeTitle": "Spec",
            "nodeKind": "video",
            "chunkKind": "transcript",
            "locator": {"location": {"kind": "media", "startMs": 65_000, "endMs": 72_000}},
            "text": "Step one."
        })],
    )
    .unwrap();
    assert!(input.system_prompt.contains("[C1]"));
    assert!(input.system_prompt.contains("human-readable title"));
    assert!(input.system_prompt.contains("`# Title`"));
    assert!(input.text.contains("cite=\"C1\""));
    assert!(input.text.contains("title=\"Spec\""));
    assert!(input.text.contains("location=\"01:05-01:12\""));
    assert!(input.text.contains("Step one."));
}

#[test]
fn a_document_with_nothing_selected_is_written_from_its_purpose_and_cites_nothing() {
    let input = request("Launch plan", &[]).unwrap();
    assert!(input.system_prompt.contains("Do not invent citations"));
    assert!(input.system_prompt.contains("`# Title`"));
    assert!(input.instruction.ends_with("Launch plan"));
    assert!(input.text.is_empty());
}

#[test]
fn promotes_generated_leading_h1_to_node_title_and_compacts_body_citations() {
    let sources = vec![
        json!({"spaceId": "s1", "nodeId": "n1", "chunkId": "k1", "nodeTitle": "First", "sourceRevision": "r1", "locator": {"sourceRevision": "r1", "location": {"kind": "node"}}, "text": "One"}),
        json!({"spaceId": "s1", "nodeId": "n2", "chunkId": "k2", "nodeTitle": "Second", "sourceRevision": "r2", "locator": {"sourceRevision": "r2", "location": {"kind": "node"}}, "text": "Two"}),
    ];
    let raw = "# Harry Potter 访谈内容研究报告 [C1]\n\n正文结论来自第二条来源 [C2]。";
    let (title, text, citations) = finalize_generated_document(raw, &sources, "s1").unwrap();
    assert_eq!(title.as_deref(), Some("Harry Potter 访谈内容研究报告"));
    assert_eq!(text, "正文结论来自第二条来源 [C1]。");
    assert_eq!(citations.as_array().unwrap().len(), 1);
    assert_eq!(citations[0]["nodeTitle"], "Second");

    let mut payload = json!({
        "kind": "markdown",
        "title": "请根据当前选中的资料，生成一份《Harry Potter 访谈内容研究报告》。",
        "text": "",
        "derivationState": "pending"
    });
    apply_document(&mut payload, title.as_deref(), &text).unwrap();
    assert_eq!(payload["title"], "Harry Potter 访谈内容研究报告");
    assert_eq!(payload["text"], "正文结论来自第二条来源 [C1]。");
    assert!(payload.get("derivationState").is_none());
}

#[test]
fn keeps_reserved_title_when_generated_markdown_has_no_leading_h1() {
    let (title, text, _) = finalize_generated_document("## Overview\n\nBody.", &[], "s1").unwrap();
    assert!(title.is_none());
    let mut payload = json!({
        "kind": "markdown",
        "title": "Reserved Title",
        "text": "",
        "derivationState": "pending"
    });
    apply_document(&mut payload, title.as_deref(), &text).unwrap();
    assert_eq!(payload["title"], "Reserved Title");
    assert_eq!(payload["text"], "## Overview\n\nBody.");
}

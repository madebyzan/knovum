#[test]
fn attached_notes_use_canonical_editor_content_and_respect_the_read_budget() {
    let data = serde_json::json!({
        "kind":"note", "knowledgeRevision":"node1:test", "plainText":"画布里的资料",
        "tiptapContent":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"画布里的资料"}]}]},
    });
    assert_eq!(super::excerpt(&data, 100).unwrap(), "画布里的资料");
    assert_eq!(super::excerpt(&data, 2).unwrap(), "画布");
}

use super::derived_chunks;
use serde_json::json;

#[test]
fn markdown_offsets_follow_crlf_utf16_positions() {
    let data = json!({"kind":"markdown","knowledgeRevision":"node1:test","text":"# A\r\n😀 body\r\n# B\r\nnext"});
    let chunks = derived_chunks(&data).unwrap();
    assert!(chunks.len() >= 2);
    assert!(chunks
        .iter()
        .all(|chunk| chunk.locator["sourceRevision"] == "node1:test"));
}

#[test]
fn section_without_knowledge_revision_has_no_derived_chunks() {
    let data = json!({"kind":"section","title":"Research"});
    assert!(derived_chunks(&data).unwrap().is_empty());
}

#[test]
fn image_visual_knowledge_generates_derived_chunk() {
    let data = json!({
        "kind":"image",
        "title":"Architecture",
        "fileName":"arch.png",
        "assetPath":"/spaces/arch.png",
        "knowledgeRevision":"node1:visual",
        "visualKnowledge": {
            "sourceRevision":"sha256:asset",
            "description":"A system architecture diagram",
            "detectedText":"Client -> API -> DB",
            "tags":["architecture","diagram"],
            "generatedAt":1,
            "provider":"test",
            "model":"vision"
        }
    });
    let chunks = derived_chunks(&data).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].kind, "image_description");
    assert_eq!(chunks[0].locator["sourceRevision"], "node1:visual");
    assert_eq!(chunks[0].locator["location"]["kind"], "node");
    assert_eq!(chunks[0].metadata["assetSourceRevision"], "sha256:asset");
    assert!(chunks[0].text.contains("Client -> API -> DB"));
}

#[test]
fn video_visual_knowledge_uses_node_locator() {
    let data = json!({
        "kind":"video",
        "title":"Demo",
        "fileName":"demo.mp4",
        "assetPath":"/spaces/demo.mp4",
        "knowledgeRevision":"node1:video",
        "visualKnowledge": {
            "sourceRevision":"sha256:video",
            "description":"A product demonstration",
            "tags":["demo"],
            "generatedAt":1,
            "provider":"test",
            "model":"vision"
        }
    });
    let chunks = derived_chunks(&data).unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].kind, "video_first_frame");
    assert_eq!(chunks[0].locator["location"]["kind"], "node");
}

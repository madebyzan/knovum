use serde_json::json;

#[test]
fn media_passages_are_generated_in_rust_with_bounded_metadata() {
    let data = json!({
        "mediaKnowledge": {
            "sourceRevision": "sha256:source",
            "schemaVersion": 3,
            "transcript": {
                "language":"en", "durationMs":1000, "text":"hello", "waveform":[],
                "segments":[{"startMs":0,"endMs":1000,"text":"hello"}],
                "subtitleCues":[{"startMs":0,"endMs":1000,"text":"hello"}]
            },
            "insights": {
                "summary":"summary", "chapters":[{"startMs":0,"endMs":1000,"title":"Intro","summary":"chapter"}],
                "promptVersion":1,"generatedAt":1,"provider":"test","model":"test"
            }
        }
    });
    let chunks = super::chunks(&data).unwrap();
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].metadata["role"], "summary");
    assert_eq!(chunks[1].kind, "chapter");
    assert_eq!(chunks[2].metadata["role"], "transcript");
    assert!(chunks
        .iter()
        .all(|chunk| chunk.locator["sourceRevision"] == "sha256:source"));
}

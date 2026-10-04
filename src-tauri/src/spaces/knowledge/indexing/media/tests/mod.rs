use serde_json::json;

use crate::media::transcript::{tests::knowledge_fixture, TranscriptSegment};

#[test]
fn media_passages_are_generated_in_rust_with_bounded_metadata() {
    let mut knowledge = knowledge_fixture(
        "sha256:source",
        vec![TranscriptSegment {
            start_ms: 0,
            end_ms: 1_000,
            text: "hello".into(),
        }],
    );
    knowledge["insights"] = json!({
        "summary":"summary",
        "chapters":[{"startMs":0,"endMs":1000,"title":"Intro","summary":"chapter"}],
        "promptVersion":1,
        "generatedAt":1,
        "executionSource":{"provider":"test","model":"test"}
    });
    let transcript_id = knowledge["transcript"]["id"].clone();
    let segment_id = knowledge["transcript"]["segments"][0]["id"].clone();
    let data = json!({"mediaKnowledge": knowledge});
    let chunks = super::chunks(&data).unwrap();
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].metadata["role"], "summary");
    assert_eq!(chunks[1].kind, "chapter");
    assert_eq!(chunks[2].metadata["role"], "transcript");
    assert_eq!(chunks[2].metadata["transcriptId"], transcript_id);
    assert_eq!(chunks[2].metadata["transcriptRevision"], 1);
    assert_eq!(chunks[2].metadata["segmentId"], segment_id);
    assert_eq!(chunks[2].metadata["segmentRevision"], 1);
    assert!(chunks
        .iter()
        .all(|chunk| chunk.locator["sourceRevision"] == "sha256:source"));
}

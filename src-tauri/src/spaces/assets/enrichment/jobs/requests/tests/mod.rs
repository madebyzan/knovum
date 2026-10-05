use serde_json::json;

use super::has_current_transcript;
use crate::media::transcript::{tests::knowledge_fixture, TranscriptSegment};

#[test]
fn current_v5_transcript_blocks_implicit_retranscription() {
    let mut data = json!({"kind":"audio","sourceRevision":"source"});
    data["mediaKnowledge"] = knowledge_fixture(
        "source",
        vec![TranscriptSegment {
            start_ms: 0,
            end_ms: 1_000,
            text: "edited transcript".into(),
        }],
    );

    assert!(has_current_transcript(&data, "source"));
    assert!(!has_current_transcript(&data, "replacement"));
}

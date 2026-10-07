use serde_json::{json, Value};

use super::{TranscriptProvenance, TranscriptSegment};
use crate::ai::contracts::speech_wire::{SpeechTranscriptArtifact, SpeechTranscriptSegment};

pub(crate) fn knowledge_fixture(source_revision: &str, segments: Vec<TranscriptSegment>) -> Value {
    let duration_ms = segments.last().map_or(0, |segment| segment.end_ms);
    let artifact = SpeechTranscriptArtifact {
        language: "en".into(),
        duration_ms,
        speech_duration_ms: duration_ms,
        segments: segments
            .into_iter()
            .map(|segment| SpeechTranscriptSegment {
                start_ms: segment.start_ms,
                end_ms: segment.end_ms,
                text: segment.text,
                words: Vec::new(),
            })
            .collect(),
    };
    let transcript = super::from_speech(
        artifact,
        duration_ms,
        source_revision,
        TranscriptProvenance {
            engine_id: "faster-whisper".into(),
            model_spec_id: "whisper-small-v1".into(),
            model_revision: "test-model-revision".into(),
            timing: "segment_only".into(),
        },
    )
    .unwrap();
    json!({"schemaVersion":1,"sourceRevision":source_revision,"transcript":transcript})
}

use super::*;
#[test]
fn exports_exact_timings_and_escaped_webvtt_speaker() {
    let cue = SpeechSegment {
        id: "cue".into(),
        ordinal: 0,
        chapter_key: None,
        speaker_key: Some("Alice & Bob".into()),
        text: "Hello <world>".into(),
        normalized_text: "Hello <world>".into(),
        voice_revision_id: "voice".into(),
        language: "en".into(),
        target_start_ms: Some(1250),
        target_end_ms: Some(3500),
        source_segment_id: None,
        source_segment_revision: None,
        source_text: None,
        state: "pending".into(),
        state_reason: None,
    };
    assert_eq!(encode(&[cue], true).unwrap(), "WEBVTT\n\n1\n00:00:01.250 --> 00:00:03.500\n<v Alice &amp; Bob>Hello &lt;world&gt;</v>\n\n");
}

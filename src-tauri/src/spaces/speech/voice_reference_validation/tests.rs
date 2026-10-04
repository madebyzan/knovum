use super::*;
use crate::ai::contracts::speech_wire::SpeechTranscriptSegment;

fn artifact(
    language: &str,
    duration_ms: u64,
    speech_duration_ms: u64,
    text: &str,
) -> SpeechTranscriptArtifact {
    SpeechTranscriptArtifact {
        language: language.into(),
        duration_ms,
        speech_duration_ms,
        segments: vec![SpeechTranscriptSegment {
            start_ms: 0,
            end_ms: speech_duration_ms,
            text: text.into(),
            words: Vec::new(),
        }],
    }
}

#[test]
fn vad_requires_real_speech_at_the_sixty_percent_boundary() {
    validate_artifact(&artifact("en", 5_000, 3_000, "hello"), "en").unwrap();
    assert!(
        validate_artifact(&artifact("en", 5_000, 2_999, "hello"), "en")
            .unwrap_err()
            .contains("at least 60%")
    );
    validate_artifact(&artifact("en", 30_000, 18_000, "hello"), "en").unwrap();
}

#[test]
fn vad_rejects_out_of_bounds_and_impossible_durations() {
    for value in [
        artifact("en", 4_999, 4_999, "hello"),
        artifact("en", 30_001, 18_001, "hello"),
        artifact("en", 5_000, 5_001, "hello"),
    ] {
        assert!(validate_artifact(&value, "en")
            .unwrap_err()
            .contains("invalid duration"));
    }
}

#[test]
fn transcript_match_normalizes_case_and_punctuation() {
    let value = artifact("en", 5_000, 4_000, "Hello, WORLD 2026!");
    validate_confirmed_text("hello world 2026", &value).unwrap();
}

#[test]
fn transcript_match_enforces_the_twenty_percent_distance_boundary() {
    validate_confirmed_text("abcdefghij", &artifact("en", 5_000, 4_000, "abXdefghiY")).unwrap();
    assert!(
        validate_confirmed_text("abcdefghij", &artifact("en", 5_000, 4_000, "abXdeYghiZ"),)
            .unwrap_err()
            .contains("more than 20%")
    );
}

#[test]
fn transcript_match_rejects_empty_and_unbounded_inputs() {
    assert!(
        validate_confirmed_text("...", &artifact("en", 5_000, 4_000, "hello"))
            .unwrap_err()
            .contains("could not be matched")
    );
    let oversized = "a".repeat(MAX_MATCH_CHARACTERS + 1);
    assert!(
        validate_confirmed_text(&oversized, &artifact("en", 5_000, 4_000, "hello"))
            .unwrap_err()
            .contains("could not be matched")
    );
}

#[test]
fn language_policy_accepts_only_supported_exact_mappings() {
    validate_artifact(&artifact("zh", 5_000, 4_000, "你好"), "zh-CN").unwrap();
    validate_artifact(&artifact("zh-CN", 5_000, 4_000, "你好"), "zh-CN").unwrap();
    validate_artifact(&artifact("en", 5_000, 4_000, "hello"), "en").unwrap();

    for (detected, expected) in [("zh-TW", "zh-CN"), ("en-US", "en"), ("en", "fr")] {
        assert!(
            validate_artifact(&artifact(detected, 5_000, 4_000, "speech"), expected)
                .unwrap_err()
                .contains("language was detected")
        );
    }
}

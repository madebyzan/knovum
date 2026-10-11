use serde_json::{json, Value};

use crate::media::transcript::{tests::knowledge_fixture, TranscriptSegment};

fn valid_media_knowledge() -> Value {
    knowledge_fixture(
        "sha256:source",
        vec![TranscriptSegment {
            start_ms: 100,
            end_ms: 1_100,
            text: "Hello world".into(),
        }],
    )
}

#[test]
fn accepts_normalized_v1_transcript() {
    super::validate(Some(&valid_media_knowledge()), Some("sha256:source"))
        .expect("valid v1 media transcript");
}

#[test]
fn accepts_persistent_ids_independent_of_current_positions() {
    let mut knowledge = valid_media_knowledge();
    knowledge["transcript"]["id"] = json!("e".repeat(64));
    knowledge["transcript"]["segments"][0]["id"] = json!("f".repeat(64));
    knowledge["transcript"]["segments"][0]["units"][0]["id"] = json!("a".repeat(64));
    super::validate(Some(&knowledge), Some("sha256:source"))
        .expect("valid persistent ids are not recomputed by the boundary validator");
}

#[test]
fn rejects_unsupported_transcript_versions() {
    let mut knowledge = valid_media_knowledge();
    for unsupported in [0, 2, 4, 5] {
        knowledge["schemaVersion"] = json!(unsupported);
        assert_eq!(
            super::validate(Some(&knowledge), Some("sha256:source")).unwrap_err(),
            "mediaKnowledge schemaVersion must be 1"
        );
    }
}

#[test]
fn rejects_source_revision_mismatch() {
    let knowledge = valid_media_knowledge();
    assert_eq!(
        super::validate(Some(&knowledge), Some("sha256:other")).unwrap_err(),
        "mediaKnowledge sourceRevision must match node sourceRevision"
    );
}

#[test]
fn accepts_locally_edited_unaligned_segment() {
    let mut knowledge = valid_media_knowledge();
    let segment_id = knowledge["transcript"]["segments"][0]["id"]
        .as_str()
        .expect("segment id")
        .to_owned();
    knowledge["transcript"]["revision"] = json!(2);
    knowledge["transcript"]["text"] = json!("Corrected text");
    knowledge["transcript"]["segments"][0]["revision"] = json!(2);
    knowledge["transcript"]["segments"][0]["text"] = json!("Corrected text");
    knowledge["transcript"]["segments"][0]["units"] = json!([{
        "id": segment_id,
        "text": "Corrected text",
        "startCharacter": 0,
        "endCharacter": 14,
        "startMs": null,
        "endMs": null,
        "score": null,
        "unalignedReason": "text_edited",
        "unitKind": "segment"
    }]);
    knowledge["transcript"]["subtitleCues"] =
        json!([{"startMs":100,"endMs":1_100,"text":"Corrected text"}]);
    super::validate(Some(&knowledge), Some("sha256:source"))
        .expect("locally edited transcript remains valid");
}

#[test]
fn rejects_inexact_16khz_sample_offsets() {
    let mut knowledge = valid_media_knowledge();
    knowledge["transcript"]["segments"][0]["startSample"] = json!(1_601);
    assert!(super::validate(Some(&knowledge), Some("sha256:source"))
        .unwrap_err()
        .contains("16 kHz timeline"));
}

#[test]
fn rejects_unknown_normalized_transcript_fields() {
    let mut knowledge = valid_media_knowledge();
    knowledge["transcript"]["legacyTiming"] = json!(true);
    assert!(super::validate(Some(&knowledge), Some("sha256:source"))
        .unwrap_err()
        .contains("unknown field `legacyTiming`"));
}

fn aligned_hello_world_with_gap() -> Value {
    let mut knowledge = valid_media_knowledge();
    knowledge["transcript"]["revision"] = json!(2);
    knowledge["transcript"]["alignment"] = json!({
        "engineId": "qwen3_forced_aligner",
        "modelSpecId": "qwen3-forced-aligner-0.6b-f16-cpu-v1",
        "modelRevision": "pinned-revision",
        "alignedAt": 1
    });
    knowledge["transcript"]["segments"][0]["units"] = json!([{
        "id": "a".repeat(64),
        "text": "Hello",
        "startCharacter": 0,
        "endCharacter": 5,
        "startMs": 100,
        "endMs": 400,
        "score": 0.99,
        "unalignedReason": null,
        "unitKind": "word"
    }, {
        "id": "b".repeat(64),
        "text": " ",
        "startCharacter": 5,
        "endCharacter": 6,
        "startMs": null,
        "endMs": null,
        "score": null,
        "unalignedReason": "not_aligned",
        "unitKind": "segment"
    }, {
        "id": "c".repeat(64),
        "text": "world",
        "startCharacter": 6,
        "endCharacter": 11,
        "startMs": 500,
        "endMs": 900,
        "score": 0.98,
        "unalignedReason": null,
        "unitKind": "word"
    }]);
    knowledge
}

#[test]
fn accepts_alignment_gap_unit_with_raw_space_text() {
    let knowledge = aligned_hello_world_with_gap();
    super::validate(Some(&knowledge), Some("sha256:source"))
        .expect("alignment gaps preserve source whitespace");
}

#[test]
fn rejects_empty_alignment_unit_text() {
    let mut knowledge = aligned_hello_world_with_gap();
    knowledge["transcript"]["segments"][0]["units"][1]["text"] = json!("");
    assert_eq!(
        super::validate(Some(&knowledge), Some("sha256:source")).unwrap_err(),
        "Transcript unit text is invalid"
    );
}

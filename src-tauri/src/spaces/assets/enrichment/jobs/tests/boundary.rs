use serde_json::json;

#[test]
fn media_publication_rejects_a_concurrently_edited_transcript() {
    let expected = json!({"transcript":{"text":"original"}});
    let current = json!({"transcript":{"text":"corrected"}});
    assert!(
        crate::spaces::assets::enrichment::media_knowledge::validate_input(&current, &expected)
            .is_err()
    );
    assert!(
        crate::spaces::assets::enrichment::media_knowledge::validate_input(&expected, &expected)
            .is_ok()
    );
}

#[test]
fn obsolete_embedding_jobs_are_not_executable_contract_variants() {
    for value in ["embed_text", "embed_media_transcript"] {
        assert!(
            serde_json::from_value::<super::super::job::EnrichmentJobType>(json!(value)).is_err()
        );
    }
}

#[test]
fn obsolete_visual_jobs_are_rejected_instead_of_aliased() {
    for value in ["analyze_image", "analyze_video_cover"] {
        assert!(
            serde_json::from_value::<super::super::job::EnrichmentJobType>(json!(value)).is_err()
        );
    }
    assert_eq!(
        serde_json::from_value::<super::super::job::EnrichmentJobType>(json!("analyze_visual"))
            .unwrap(),
        super::super::job::EnrichmentJobType::AnalyzeVisual,
    );
}

#[test]
fn chapters_must_use_unique_transcript_timestamps() {
    use crate::spaces::assets::enrichment::media_knowledge::insights::chapters;
    let transcript = json!({"durationMs":2000,"segments":[{"startMs":0,"text":"a"},{"startMs":1000,"text":"b"}]});
    let valid =
        json!([{"startMs":0,"title":"A","summary":"a"},{"startMs":1000,"title":"B","summary":"b"}]);
    assert_eq!(chapters(&valid, &transcript).unwrap()[0]["endMs"], 1000);
    let invalid = json!([{"startMs":500,"title":"A","summary":"a"}]);
    assert!(chapters(&invalid, &transcript).is_err());
    let duplicate =
        json!([{"startMs":0,"title":"A","summary":"a"},{"startMs":0,"title":"B","summary":"b"}]);
    assert!(chapters(&duplicate, &transcript).is_err());
}

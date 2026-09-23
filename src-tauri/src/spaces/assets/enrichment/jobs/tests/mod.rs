use super::job::*;
use serde_json::json;

#[test]
fn job_type_maps_to_snake_case_strings() {
    assert_eq!(EnrichmentJobType::EmbedText.as_str(), "embed_text");
    assert_eq!(EnrichmentJobType::AnalyzeImage.as_str(), "analyze_image");
    assert_eq!(
        EnrichmentJobType::TranscribeMedia.as_str(),
        "transcribe_media"
    );
    assert_eq!(
        EnrichmentJobType::AnalyzeVideoCover.as_str(),
        "analyze_video_cover",
    );
    assert_eq!(
        EnrichmentJobType::EmbedMediaTranscript.as_str(),
        "embed_media_transcript",
    );
}

#[test]
fn preview_failure_does_not_block_independent_enrichment_jobs() {
    let data = json!({"sourceRevision": "revision-1", "processingState": "failed"});

    assert_eq!(super::revision::source_revision(&data), Some("revision-1"));
}

#[test]
fn unfinished_asset_copy_does_not_schedule_enrichment_jobs() {
    let data = json!({"sourceRevision": "revision-1", "processingState": "processing"});

    assert_eq!(super::revision::source_revision(&data), None);
}

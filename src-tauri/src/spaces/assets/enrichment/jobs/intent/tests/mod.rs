use super::*;
use serde_json::json;

#[test]
fn image_analysis_uses_rendered_revision_and_reuses_current_knowledge() {
    let mut data = json!({"kind":"image","sourceRevision":"asset-1","imageEdit":null});
    let revision =
        super::super::revision::expected_revision(&data, EnrichmentJobType::AnalyzeVisual)
            .unwrap()
            .unwrap();
    assert_ne!(revision, "asset-1");
    assert_eq!(
        required_jobs(&data, Some(&revision), "asset-1"),
        vec![EnrichmentJobType::AnalyzeVisual]
    );
    data["visualKnowledge"] = json!({"sourceRevision":revision,"description":"A diagram"});
    assert!(required_jobs(&data, Some(&revision), "asset-1").is_empty());
}

#[test]
fn video_without_cover_only_schedules_media_analysis() {
    let data = json!({"kind":"video","sourceRevision":"asset-1"});
    assert_eq!(
        required_jobs(&data, visual_revision(&data).unwrap().as_deref(), "asset-1"),
        vec![EnrichmentJobType::AnalyzeMedia]
    );
}

#[test]
fn video_cover_analysis_does_not_replace_media_analysis() {
    let data =
        json!({"kind":"video","sourceRevision":"asset-1","previewPath":"/previews/cover.png"});
    assert_eq!(
        required_jobs(&data, visual_revision(&data).unwrap().as_deref(), "asset-1"),
        vec![
            EnrichmentJobType::AnalyzeMedia,
            EnrichmentJobType::AnalyzeVisual
        ]
    );
}

#[test]
fn replacing_a_cover_invalidates_visual_knowledge_but_not_the_transcript() {
    let mut data = json!({
        "kind":"video","sourceRevision":"asset-1","previewPath":"/previews/first.png",
        "mediaAnalysis":{"sourceRevision":"asset-1"},
        "mediaKnowledge":{"sourceRevision":"asset-1"}
    });
    let first = visual_revision(&data).unwrap().unwrap();
    data["visualKnowledge"] = json!({"sourceRevision":first,"description":"First frame"});
    assert!(required_jobs(&data, Some(&first), "asset-1").is_empty());
    data["previewPath"] = json!("/previews/second.png");
    let second = visual_revision(&data).unwrap().unwrap();
    assert_ne!(first, second);
    assert_eq!(
        required_jobs(&data, Some(&second), "asset-1"),
        vec![EnrichmentJobType::AnalyzeVisual]
    );
}

#[test]
fn visual_revision_ignores_titles_but_tracks_the_video_source() {
    let mut data =
        json!({"kind":"video","sourceRevision":"asset-1","previewPath":"/previews/first.png"});
    let first = visual_revision(&data).unwrap();
    data["title"] = json!("Renamed");
    assert_eq!(visual_revision(&data).unwrap(), first);
    data["origin"] = json!({"kind":"web"});
    assert_ne!(visual_revision(&data).unwrap(), first);
    data.as_object_mut().unwrap().remove("origin");
    data["sourceRevision"] = json!("asset-2");
    assert_ne!(visual_revision(&data).unwrap(), first);
}

mod persistence;

#[test]
fn pdf_resources_are_materialized_before_visual_analysis() {
    let data = json!({
        "kind": "file", "fileName": "report.pdf", "sourceRevision": "revision-1",
        "documentCoverage": { "sourceRevision": "revision-1", "occurrences": [{
            "pdfInput": { "kind": "image", "page": 1 }, "resourceId": null, "state": "pending"
        }]}
    });
    assert_eq!(
        document_jobs(&data, "revision-1"),
        vec![EnrichmentJobType::ParseDocument]
    );
}

#[test]
fn materialized_document_resources_can_advance_to_visual_analysis() {
    let data = json!({
        "kind": "file", "fileName": "report.pdf", "sourceRevision": "revision-1",
        "documentCoverage": { "sourceRevision": "revision-1", "occurrences": [{
            "pdfInput": { "kind": "image", "page": 1 }, "resourceId": "resource-1", "state": "pending"
        }]}
    });
    assert_eq!(
        document_jobs(&data, "revision-1"),
        vec![EnrichmentJobType::AnalyzeDocument]
    );
}

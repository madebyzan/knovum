use super::*;
use serde_json::json;

fn owed(data: &Value) -> Vec<EnrichmentJobType> {
    required_jobs(data)
        .unwrap()
        .into_iter()
        .map(|(job_type, _)| job_type)
        .collect()
}

#[test]
fn image_analysis_uses_rendered_revision_and_reuses_current_knowledge() {
    let mut data = json!({"kind":"image","sourceRevision":"asset-1","imageEdit":null});
    let revision =
        super::super::revision::expected_revision(&data, EnrichmentJobType::AnalyzeVisual)
            .unwrap()
            .unwrap();
    assert_ne!(revision, "asset-1");
    data["textRecognition"] = json!({"sourceRevision":revision});
    assert_eq!(owed(&data), vec![EnrichmentJobType::AnalyzeVisual]);
    data["visualKnowledge"] = json!({"sourceRevision":revision,"description":"A diagram"});
    assert!(owed(&data).is_empty());
}

#[test]
fn video_without_cover_only_schedules_media_analysis() {
    let data = json!({"kind":"video","sourceRevision":"asset-1"});
    assert_eq!(owed(&data), vec![EnrichmentJobType::AnalyzeMedia]);
}

#[test]
fn a_video_cover_is_read_and_analyzed_beside_media_analysis() {
    let data =
        json!({"kind":"video","sourceRevision":"asset-1","previewPath":"/previews/cover.png"});
    let cover = visual_revision(&data).unwrap().unwrap();
    assert_eq!(
        required_jobs(&data).unwrap(),
        vec![
            (EnrichmentJobType::AnalyzeMedia, "asset-1".to_owned()),
            (EnrichmentJobType::RecognizeText, cover.clone()),
            (EnrichmentJobType::AnalyzeVisual, cover),
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
    data["textRecognition"] = json!({"sourceRevision":first});
    assert!(owed(&data).is_empty());
    data["previewPath"] = json!("/previews/second.png");
    let second = visual_revision(&data).unwrap().unwrap();
    assert_ne!(first, second);
    assert_eq!(
        owed(&data),
        vec![
            EnrichmentJobType::RecognizeText,
            EnrichmentJobType::AnalyzeVisual
        ]
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

#[test]
fn recognition_follows_a_parsed_document_once_per_revision() {
    let mut data = json!({
        "kind": "file", "fileName": "scan.pdf", "sourceRevision": "revision-1",
        "documentCoverage": {
            "sourceRevision": "revision-1",
            "resources": [{ "id": "page", "path": "/assets/page.png" }],
            "occurrences": [{ "location": { "kind": "pdf", "page": 1 }, "resourceId": "page" }]
        }
    });
    assert_eq!(
        required_jobs(&data).unwrap(),
        vec![(EnrichmentJobType::RecognizeText, "revision-1".to_owned())]
    );
    data["textRecognition"] = json!({ "sourceRevision": "revision-1" });
    assert!(owed(&data).is_empty());
    data["textRecognition"]["sourceRevision"] = json!("revision-0");
    assert_eq!(owed(&data), vec![EnrichmentJobType::RecognizeText]);
}

#[test]
fn an_image_is_read_again_when_an_edit_changes_what_it_shows() {
    let mut data =
        json!({ "kind": "image", "sourceRevision": "asset-1", "assetPath": "/assets/shot.png" });
    let shown = visual_revision(&data).unwrap().unwrap();
    assert_eq!(
        required_jobs(&data).unwrap(),
        vec![
            (EnrichmentJobType::RecognizeText, shown.clone()),
            (EnrichmentJobType::AnalyzeVisual, shown.clone()),
        ]
    );
    data["textRecognition"] = json!({ "sourceRevision": shown });
    data["visualKnowledge"] = json!({ "sourceRevision": shown, "description": "A receipt" });
    assert!(owed(&data).is_empty());
    data["imageEdit"] = crate::images::quarter_turn_edit();
    let edited = visual_revision(&data).unwrap().unwrap();
    assert_ne!(edited, shown, "the rendered picture changed");
    assert_eq!(
        owed(&data),
        vec![
            EnrichmentJobType::RecognizeText,
            EnrichmentJobType::AnalyzeVisual
        ]
    );
}

mod persistence;

#[test]
fn a_document_is_parsed_once_per_revision() {
    let mut data =
        json!({"kind": "file", "fileName": "report.pdf", "sourceRevision": "revision-1"});
    assert_eq!(
        document_jobs(&data, "revision-1"),
        vec![EnrichmentJobType::ParseDocument]
    );
    data["documentCoverage"] = json!({ "sourceRevision": "revision-1", "occurrences": [{
        "pdfInput": { "kind": "image", "page": 1 }, "resourceId": "resource-1", "unavailable": null
    }]});
    assert!(document_jobs(&data, "revision-1").is_empty());
    assert_eq!(
        document_jobs(&data, "revision-2"),
        vec![EnrichmentJobType::ParseDocument]
    );
}

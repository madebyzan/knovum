use super::*;
use serde_json::json;

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

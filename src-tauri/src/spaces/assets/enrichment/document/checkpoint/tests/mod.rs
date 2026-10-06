use super::*;
use crate::spaces::knowledge::node_contracts::document_coverage::{DocumentAnalysis, VisualState};

fn occurrence(description: &str, state: VisualState) -> DocumentOccurrence {
    DocumentOccurrence {
        id: "region".into(),
        location: serde_json::json!({"kind":"word","section":0,"element":1,"headingPath":[]}),
        context: "Body".into(),
        alt_text: None,
        resource_id: Some("raster".into()),
        pdf_input: None,
        state,
        error: None,
        analysis: Some(DocumentAnalysis {
            description: description.into(),
            detected_text: String::new(),
            execution_source: crate::ai::tasks::ExecutionSource::Agent {
                runtime_kind: crate::ai::contracts::ExternalAgentKind::Codex,
            },
            generated_at: 1,
            analysis_version: "document-visual:1".into(),
        }),
    }
}

#[test]
fn recovery_preserves_published_edits_and_resumes_unpublished_work() {
    let checkpoint = occurrence("Old result", VisualState::Completed);
    let mut published = occurrence("Human edit", VisualState::Completed);
    restore_occurrence(&mut published, checkpoint.clone()).unwrap();
    assert_eq!(published.analysis.unwrap().description, "Human edit");
    let mut pending = occurrence("", VisualState::Pending);
    restore_occurrence(&mut pending, checkpoint.clone()).unwrap();
    assert_eq!(pending.analysis.unwrap().description, "Old result");
    let mut changed = occurrence("", VisualState::Pending);
    changed.location = serde_json::json!({"kind":"word","section":1,"element":1,"headingPath":[]});
    assert!(restore_occurrence(&mut changed, checkpoint).is_err());
}

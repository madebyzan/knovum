use crate::spaces::knowledge::indexing::{self, KnowledgeChunkSeed};
use crate::spaces::knowledge::model::document_coverage::{DocumentCoverage, VisualState};
use serde_json::{json, Value};

pub(in crate::spaces) fn chunks(payload: &Value) -> Result<Vec<KnowledgeChunkSeed>, String> {
    if payload.get("documentCoverage").is_none() {
        return Ok(Vec::new());
    }
    let report = DocumentCoverage::from_node_content(payload)?;
    let revision = payload
        .get("knowledgeRevision")
        .and_then(Value::as_str)
        .ok_or("Document visual chunks require knowledgeRevision")?;
    let mut chunks = Vec::new();
    for occurrence in report.occurrences {
        if occurrence.state != VisualState::Completed {
            continue;
        }
        let analysis = occurrence
            .analysis
            .ok_or("Completed document occurrence has no analysis")?;
        for (kind, role, text) in [
            ("text", "document_ocr", analysis.detected_text),
            (
                "image_description",
                "document_interpretation",
                analysis.description,
            ),
        ] {
            if text.trim().is_empty() {
                continue;
            }
            chunks.extend(indexing::chunks(kind, &text, revision, occurrence.location.clone(), json!({
                "role":role, "occurrenceId":occurrence.id, "assetSourceRevision":report.source_revision,
                "executionEngine":analysis.execution_source.engine(), "analysisVersion":analysis.analysis_version,
                "context":occurrence.context,
            })));
        }
    }
    Ok(chunks)
}

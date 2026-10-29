//! Evidence spans as Provenance.
//!
//! Analysis creates Provenance only from spans it has proven, so every quote a proposal records
//! exists at the revision it names. The locator is the passage's own, with the exact quote added.

use serde_json::json;

use crate::spaces::knowledge::{
    change_commit::KnowledgeChange, grounding::evidence_span::EvidenceSpan,
};

/// The operation that records `span` as evidence for the node `target_id` names.
pub(crate) fn provenance(span: &EvidenceSpan, target_id: &str) -> KnowledgeChange {
    let mut locator = span.locator.clone();
    if let Some(fields) = locator.as_object_mut() {
        fields.insert("quote".into(), json!({ "exact": span.quote }));
    }
    KnowledgeChange::AddProvenance {
        target_kind: "node".into(),
        target_id: target_id.to_owned(),
        source_node_id: span.node_id.clone(),
        locator,
        source_revision: span.source_revision.clone(),
        quote: span.quote.clone(),
    }
}

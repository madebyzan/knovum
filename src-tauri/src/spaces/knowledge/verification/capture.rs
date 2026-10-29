//! A verified answer as proposed knowledge: one Claim per chosen statement.
//!
//! A statement the sources support becomes an accepted Claim with the exact quotes as its
//! Provenance and a `supports` relation from every source it quotes. Anything less becomes a
//! draft Claim; one without evidence carries none, so Knowledge Health reports it as unsupported
//! instead of the proposal pretending otherwise. A person's correction of a verdict wins.

use std::collections::BTreeSet;

use serde_json::{json, Map};

use super::contracts::{AnswerStatement, StatementSupport, StatementVerdict, SupportCorrection};
use crate::spaces::knowledge::{
    change_commit::KnowledgeChange,
    proposals::{basis, ProposalDraft, ProposalOrigin},
};

const CLAIM_TITLE_CHARS: usize = 200;

pub(crate) struct VerifiedAnswer<'a> {
    pub conversation_id: &'a str,
    pub message_id: &'a str,
    pub statements: &'a [AnswerStatement],
    pub verdicts: &'a [StatementVerdict],
    pub corrections: &'a [SupportCorrection],
}

impl VerifiedAnswer<'_> {
    /// The support a person sees: their correction when they made one, otherwise the verdict.
    pub(crate) fn support(&self, statement: u32) -> Option<StatementSupport> {
        self.corrections
            .iter()
            .find(|correction| correction.statement == statement)
            .map(|correction| correction.support)
            .or_else(|| self.verdict(statement).map(|verdict| verdict.support))
    }

    fn verdict(&self, statement: u32) -> Option<&StatementVerdict> {
        self.verdicts
            .iter()
            .find(|verdict| verdict.statement == statement)
    }
}

/// Proposes the chosen statements as Claims. Statements that are not claims about the sources
/// (inference and framing) cannot be chosen.
pub(crate) fn claims_proposal(
    answer: &VerifiedAnswer,
    chosen: &[u32],
) -> Result<ProposalDraft, String> {
    let chosen: BTreeSet<u32> = chosen.iter().copied().collect();
    let mut changes = Vec::new();
    let mut basis_spans = Vec::new();
    for statement in answer.statements.iter().filter(|s| chosen.contains(&s.id)) {
        let support = answer
            .support(statement.id)
            .ok_or("answer-capture-unverified: statement has no verdict")?;
        if support == StatementSupport::NotCheckable {
            return Err(
                "answer-capture-invalid: only factual statements can be recorded as claims".into(),
            );
        }
        let evidence = answer
            .verdict(statement.id)
            .filter(|_| {
                matches!(
                    support,
                    StatementSupport::Supported | StatementSupport::Partial
                )
            })
            .map(|verdict| verdict.evidence.as_slice())
            .unwrap_or_default();
        let claim = format!("claim-{}", statement.id);
        let accepted = support == StatementSupport::Supported && !evidence.is_empty();
        changes.push(KnowledgeChange::CreateObject {
            temp_id: claim.clone(),
            object_type_id: "system:claim".into(),
            title: statement.text.chars().take(CLAIM_TITLE_CHARS).collect(),
            summary: None,
            status: if accepted { "accepted" } else { "draft" }.into(),
            properties: Map::from_iter([("statement".to_owned(), json!(statement.text))]),
            position: None,
        });
        let sources: BTreeSet<&str> = evidence.iter().map(|e| e.span.node_id.as_str()).collect();
        for (index, source) in sources.into_iter().enumerate() {
            changes.push(KnowledgeChange::CreateRelation {
                temp_id: format!("{claim}-supports-{index}"),
                source_id: source.to_owned(),
                target_id: claim.clone(),
                kind: "supports".into(),
                rationale: String::new(),
                confidence: None,
            });
        }
        for evidence in evidence {
            changes.push(basis::provenance(&evidence.span, &claim));
            basis_spans.push(evidence.span.clone());
        }
    }
    if changes.is_empty() {
        return Err("answer-capture-invalid: choose at least one statement".into());
    }
    Ok(ProposalDraft {
        origin: ProposalOrigin::Answer {
            conversation_id: answer.conversation_id.to_owned(),
            message_id: answer.message_id.to_owned(),
        },
        rationale: String::new(),
        changes,
        basis: basis_spans,
    })
}

#[cfg(test)]
#[path = "tests/capture.rs"]
mod tests;

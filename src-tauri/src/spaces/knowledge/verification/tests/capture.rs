use serde_json::json;

use super::*;
use crate::spaces::knowledge::{
    grounding::evidence_span::EvidenceSpan,
    verification::contracts::{StatementEvidence, StatementRole},
};

fn statement(id: u32, text: &str) -> AnswerStatement {
    AnswerStatement {
        id,
        text: text.into(),
        start: 0,
        end: text.len(),
        citations: vec![1],
    }
}

fn evidence(node: &str, quote: &str) -> StatementEvidence {
    StatementEvidence {
        citation: 1,
        span: EvidenceSpan {
            chunk_id: format!("{node}-chunk"),
            node_id: node.into(),
            source_revision: "rev-1".into(),
            locator: json!({"sourceRevision": "rev-1", "location": {"kind": "text"}}),
            quote_start: 0,
            quote_end: quote.len(),
            quote: quote.into(),
        },
    }
}

fn verdict(
    statement: u32,
    support: StatementSupport,
    evidence: Vec<StatementEvidence>,
) -> StatementVerdict {
    let role = if support == StatementSupport::NotCheckable {
        StatementRole::Framing
    } else {
        StatementRole::Factual
    };
    StatementVerdict {
        statement,
        role,
        support,
        evidence,
        possible_sources: Vec::new(),
        reason: String::new(),
    }
}

fn statements() -> Vec<AnswerStatement> {
    vec![
        statement(1, "Revenue grew 12% in Q3."),
        statement(2, "Costs fell sharply."),
        statement(3, "Hope this helps."),
    ]
}

fn verdicts() -> Vec<StatementVerdict> {
    vec![
        verdict(
            1,
            StatementSupport::Supported,
            vec![
                evidence("report", "Revenue grew 12%"),
                evidence("deck", "12% growth"),
            ],
        ),
        verdict(2, StatementSupport::Unsupported, Vec::new()),
        verdict(3, StatementSupport::NotCheckable, Vec::new()),
    ]
}

fn answer<'a>(
    statements: &'a [AnswerStatement],
    verdicts: &'a [StatementVerdict],
    corrections: &'a [SupportCorrection],
) -> VerifiedAnswer<'a> {
    VerifiedAnswer {
        conversation_id: "conversation",
        message_id: "message",
        statements,
        verdicts,
        corrections,
    }
}

#[test]
fn a_supported_statement_becomes_an_accepted_claim_with_its_evidence() {
    let (statements, verdicts) = (statements(), verdicts());
    let draft = claims_proposal(&answer(&statements, &verdicts, &[]), &[1]).unwrap();
    let KnowledgeChange::CreateObject {
        status,
        title,
        properties,
        ..
    } = &draft.changes[0]
    else {
        panic!()
    };
    assert_eq!(
        (status.as_str(), title.as_str()),
        ("accepted", "Revenue grew 12% in Q3.")
    );
    assert_eq!(properties["statement"], "Revenue grew 12% in Q3.");
    let relations = draft
        .changes
        .iter()
        .filter(|c| matches!(c, KnowledgeChange::CreateRelation { .. }))
        .count();
    let provenance: Vec<_> = draft
        .changes
        .iter()
        .filter_map(|c| match c {
            KnowledgeChange::AddProvenance { locator, quote, .. } => {
                Some((locator["quote"]["exact"].clone(), quote.clone()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(relations, 2);
    assert_eq!(
        provenance[0],
        (json!("Revenue grew 12%"), "Revenue grew 12%".to_owned())
    );
    assert_eq!(draft.basis.len(), 2);
    assert_eq!(
        draft.origin,
        ProposalOrigin::Answer {
            conversation_id: "conversation".into(),
            message_id: "message".into()
        }
    );
}

#[test]
fn an_unsupported_statement_becomes_a_draft_claim_without_evidence() {
    let (statements, verdicts) = (statements(), verdicts());
    let draft = claims_proposal(&answer(&statements, &verdicts, &[]), &[2]).unwrap();
    assert_eq!(draft.changes.len(), 1);
    assert!(
        matches!(&draft.changes[0], KnowledgeChange::CreateObject { status, .. } if status == "draft")
    );
    assert!(draft.basis.is_empty());
}

#[test]
fn a_correction_wins_over_the_verdict() {
    let (statements, verdicts) = (statements(), verdicts());
    let corrections = [SupportCorrection {
        statement: 1,
        support: StatementSupport::Unsupported,
    }];
    let draft = claims_proposal(&answer(&statements, &verdicts, &corrections), &[1]).unwrap();
    assert_eq!(draft.changes.len(), 1);
    assert!(
        matches!(&draft.changes[0], KnowledgeChange::CreateObject { status, .. } if status == "draft")
    );
}

#[test]
fn framing_and_empty_choices_are_refused() {
    let (statements, verdicts) = (statements(), verdicts());
    assert!(claims_proposal(&answer(&statements, &verdicts, &[]), &[3]).is_err());
    assert!(claims_proposal(&answer(&statements, &verdicts, &[]), &[]).is_err());
}

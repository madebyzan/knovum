use serde_json::json;

use super::*;

fn statement(id: u32, text: &str, citations: &[u32]) -> AnswerStatement {
    AnswerStatement {
        id,
        text: text.into(),
        start: 0,
        end: text.len(),
        citations: citations.to_vec(),
    }
}

fn passage(citation: u32, text: &str) -> ReadPassage {
    ReadPassage {
        citation,
        chunk_id: format!("chunk-{citation}"),
        node_id: format!("node-{citation}"),
        node_title: format!("Source {citation}"),
        source_revision: "rev".into(),
        locator: json!({"kind": "text"}),
        text: text.into(),
    }
}

fn passages() -> Vec<ReadPassage> {
    vec![
        passage(1, "Revenue grew 12% in the third quarter."),
        passage(2, "The free tier allows 1,000 requests per day."),
    ]
}

fn judge(reply: serde_json::Value, statements: &[AnswerStatement]) -> Vec<StatementVerdict> {
    parse(&reply.to_string(), statements, &passages()).unwrap()
}

#[test]
fn a_quoted_cited_passage_supports_its_statement() {
    let statements = [statement(1, "Revenue grew 12% in Q3.", &[1])];
    let verdicts = judge(
        json!({"statements": [{"id": "S1", "role": "factual", "support": "supported",
            "quotes": [{"source": "C1", "quote": "Revenue grew 12% in the third quarter"}], "reason": ""}]}),
        &statements,
    );
    assert_eq!(verdicts[0].support, StatementSupport::Supported);
    assert_eq!(verdicts[0].evidence[0].citation, 1);
    assert_eq!(verdicts[0].evidence[0].span.chunk_id, "chunk-1");
}

#[test]
fn a_quote_the_passage_does_not_contain_downgrades_support() {
    let statements = [statement(1, "Revenue grew 20%.", &[1])];
    let verdicts = judge(
        json!({"statements": [{"id": "S1", "role": "factual", "support": "supported",
            "quotes": [{"source": "C1", "quote": "Revenue grew 20%"}], "reason": "made up"}]}),
        &statements,
    );
    assert_eq!(verdicts[0].support, StatementSupport::Unsupported);
    assert!(verdicts[0].evidence.is_empty());
}

#[test]
fn support_from_a_passage_it_does_not_cite_is_a_possible_source() {
    let statements = [statement(
        1,
        "The free tier allows 1,000 requests a day.",
        &[1],
    )];
    let verdicts = judge(
        json!({"statements": [{"id": "S1", "role": "factual", "support": "supported",
            "quotes": [{"source": "C2", "quote": "The free tier allows 1,000 requests per day"}],
            "reason": "C2 states it, C1 does not"}]}),
        &statements,
    );
    assert_eq!(verdicts[0].support, StatementSupport::Unsupported);
    assert_eq!(verdicts[0].possible_sources[0].citation, 2);
}

#[test]
fn an_uncited_factual_statement_is_uncited_even_when_a_source_states_it() {
    let statements = [statement(1, "Revenue grew 12%.", &[])];
    let verdicts = judge(
        json!({"statements": [{"id": "S1", "role": "factual", "support": "supported",
            "quotes": [{"source": "C1", "quote": "Revenue grew 12%"}], "reason": ""}]}),
        &statements,
    );
    assert_eq!(verdicts[0].support, StatementSupport::Uncited);
    assert_eq!(verdicts[0].possible_sources.len(), 1);
}

#[test]
fn inference_and_framing_are_not_checked() {
    let statements = [
        statement(1, "So growth is healthy.", &[]),
        statement(2, "Hope this helps!", &[]),
    ];
    let verdicts = judge(
        json!({"statements": [
            {"id": "S1", "role": "inference", "quotes": [], "reason": "conclusion"},
            {"id": "S2", "role": "framing", "quotes": [], "reason": ""}]}),
        &statements,
    );
    assert!(verdicts
        .iter()
        .all(|v| v.support == StatementSupport::NotCheckable));
}

#[test]
fn a_partial_verdict_keeps_its_evidence_and_reason() {
    let statements = [statement(1, "Revenue grew 12% because of pricing.", &[1])];
    let verdicts = judge(
        json!({"statements": [{"id": "S1", "role": "factual", "support": "partial",
            "quotes": [{"source": "C1", "quote": "Revenue grew 12%"}], "reason": "The cause is not stated."}]}),
        &statements,
    );
    assert_eq!(verdicts[0].support, StatementSupport::Partial);
    assert_eq!(verdicts[0].reason, "The cause is not stated.");
}

#[test]
fn the_reply_must_judge_exactly_the_issued_statements() {
    let statements = [
        statement(1, "One claim here.", &[1]),
        statement(2, "Two claims here.", &[1]),
    ];
    let one = json!({"statements": [{"id": "S1", "role": "framing", "quotes": [], "reason": ""}]});
    assert!(parse(&one.to_string(), &statements, &passages())
        .unwrap_err()
        .contains("S2"));
    let extra = json!({"statements": [
        {"id": "S1", "role": "framing", "quotes": [], "reason": ""},
        {"id": "S2", "role": "framing", "quotes": [], "reason": ""},
        {"id": "S9", "role": "framing", "quotes": [], "reason": ""}]});
    assert!(parse(&extra.to_string(), &statements, &passages())
        .unwrap_err()
        .contains("S9"));
    let unknown_field = json!({"statements": [], "confidence": 1});
    assert!(parse(&unknown_field.to_string(), &[], &passages()).is_err());
}

#[test]
fn the_prompt_lists_cited_passages_first_within_budget() {
    let statements = [statement(1, "Requests are limited.", &[2])];
    let text = prompt(&statements, &passages());
    assert!(text.contains("S1 (cites C2): Requests are limited."));
    assert!(text.find("C2 | Source 2").unwrap() < text.find("C1 | Source 1").unwrap());
    let large: Vec<ReadPassage> = (1..=20)
        .map(|i| passage(i, &"x".repeat(PASSAGE_CHARS)))
        .collect();
    let bounded = prompt(&statements, &large);
    assert_eq!(
        bounded.matches(" | Source ").count(),
        SOURCE_BUDGET_CHARS / PASSAGE_CHARS
    );
}

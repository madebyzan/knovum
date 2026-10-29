use serde_json::json;

use super::*;

#[test]
fn a_verbatim_quote_keeps_its_exact_bytes() {
    let text = "Revenue grew 12% in Q3. Costs were flat.";
    assert_eq!(locate(text, "Costs were flat."), Some((24, 40)));
}

#[test]
fn a_model_normalized_quote_maps_back_to_the_source_text() {
    let text = "The API is \u{201C}rate-limited\u{201D}\n   per key — not per account.";
    let (start, end) = locate(text, "the api is \"rate-limited\" per key").unwrap();
    assert_eq!(
        &text[start..end],
        "The API is \u{201C}rate-limited\u{201D}\n   per key"
    );
}

#[test]
fn chinese_punctuation_variants_still_match() {
    let text = "结论：方案（A）成本更低，部署更快。";
    let (start, end) = locate(text, "方案(A)成本更低,部署更快").unwrap();
    assert_eq!(&text[start..end], "方案（A）成本更低，部署更快");
}

#[test]
fn a_quote_the_passage_does_not_contain_is_refused() {
    assert_eq!(locate("Revenue grew 12% in Q3.", "Revenue grew 21%"), None);
    assert_eq!(locate("Revenue grew 12% in Q3.", "   "), None);
    assert_eq!(locate("short", &"x".repeat(MAX_QUOTE_BYTES + 1)), None);
}

#[test]
fn a_span_copies_identity_from_the_passage_and_text_from_the_source() {
    let locator = json!({"kind": "page", "page": 3});
    let passage = Passage {
        chunk_id: "chunk-1",
        node_id: "node-1",
        source_revision: "rev-1",
        locator: &locator,
        text: "Alpha beta GAMMA delta.",
    };
    let span = passage.span("beta gamma").unwrap();
    assert_eq!(span.quote, "beta GAMMA");
    assert_eq!((span.quote_start, span.quote_end), (6, 16));
    assert_eq!(span.chunk_id, "chunk-1");
    assert_eq!(span.locator, locator);
    assert!(passage.span("epsilon").is_none());
}

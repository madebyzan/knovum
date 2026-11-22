use serde_json::json;

use super::{finalize_citations, format_location, format_sources, strip_citation_markers};

#[test]
fn formats_media_and_document_locations() {
    assert_eq!(
        format_location(
            &json!({"location": {"kind": "media", "startMs": 98_000, "endMs": 105_000}})
        ),
        Some("01:38-01:45".into())
    );
    assert_eq!(
        format_location(&json!({"location": {"kind": "pdf", "page": 12}})),
        Some("p.12".into())
    );
    assert_eq!(
        format_location(&json!({"location": {"kind": "markdown", "heading": "Summary"}})),
        Some("Summary".into())
    );
}

#[test]
fn formats_sources_with_structured_metadata_and_location() {
    let formatted = format_sources(
        &[json!({
            "nodeTitle": "Emma Watson Interview",
            "nodeKind": "video",
            "chunkKind": "transcript",
            "locator": {"location": {"kind": "media", "startMs": 98_000, "endMs": 105_000}},
            "text": "There is so much merchandise."
        })],
        false,
    )
    .unwrap();
    assert_eq!(
        formatted,
        "<source cite=\"C1\" title=\"Emma Watson Interview\" type=\"video\" kind=\"transcript\" location=\"01:38-01:45\">\nThere is so much merchandise.\n</source>"
    );
}

#[test]
fn compacts_citations_sequentially_drops_invalid_and_skips_code() {
    let sources = vec![
        json!({"spaceId": "s1", "nodeId": "n1", "chunkId": "k1", "nodeTitle": "First", "sourceRevision": "r1", "locator": {"sourceRevision": "r1", "location": {"kind": "node"}}, "text": "One"}),
        json!({"spaceId": "s1", "nodeId": "n2", "chunkId": "k2", "nodeTitle": "Second", "sourceRevision": "r2", "locator": {"sourceRevision": "r2", "location": {"kind": "node"}}, "text": "Two"}),
        json!({"spaceId": "s1", "nodeId": "n3", "chunkId": "k3", "nodeTitle": "Third", "sourceRevision": "r3", "locator": {"sourceRevision": "r3", "location": {"kind": "node"}}, "text": "Three"}),
    ];
    let input = "Start [C3][C1], grouped [C1, C3], invalid [C99]. Keep `[C3]`.\n```md\n[C3]\n```\n";
    let (text, citations) = finalize_citations(input, &sources, "s1").unwrap();
    assert_eq!(
        text,
        "Start [C1][C2], grouped [C2][C1], invalid. Keep `[C3]`.\n```md\n[C3]\n```\n"
    );
    let list = citations.as_array().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["index"], 1);
    assert_eq!(list[0]["nodeTitle"], "Third");
    assert_eq!(list[1]["index"], 2);
    assert_eq!(list[1]["nodeTitle"], "First");
}

#[test]
fn strips_citation_markers_cleanly_while_preserving_spacing_and_code() {
    let input = "Claim [C1][C2]. Mid [C3] next. End [C4]\nKeep `[C5]`.\n```md\n[C6]\n```\n";
    assert_eq!(
        strip_citation_markers(input),
        "Claim. Mid next. End\nKeep `[C5]`.\n```md\n[C6]\n```\n"
    );
}

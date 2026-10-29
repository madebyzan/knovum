use serde_json::{json, Map};

use super::*;

fn claim(temp_id: &str, title: &str) -> KnowledgeChange {
    KnowledgeChange::CreateObject {
        temp_id: temp_id.into(),
        object_type_id: "system:claim".into(),
        title: title.into(),
        summary: None,
        status: "draft".into(),
        properties: Map::new(),
        position: None,
    }
}

fn evidence(target: &str, quote: &str) -> KnowledgeChange {
    KnowledgeChange::AddProvenance {
        target_kind: "node".into(),
        target_id: target.into(),
        source_node_id: "report".into(),
        locator: json!({}),
        source_revision: "rev-1".into(),
        quote: quote.into(),
    }
}

#[test]
fn temp_ids_order_case_and_spacing_do_not_change_the_fingerprint() {
    let one = [
        claim("t1", "Revenue grew 12%."),
        evidence("t1", "Revenue grew 12%"),
    ];
    let two = [
        evidence("x", "revenue   grew 12%"),
        claim("x", "  revenue grew 12% "),
    ];
    assert_eq!(fingerprint(&one), fingerprint(&two));
}

#[test]
fn a_different_statement_or_source_revision_is_a_different_proposal() {
    let base = fingerprint(&[
        claim("t", "Revenue grew 12%"),
        evidence("t", "Revenue grew 12%"),
    ]);
    assert_ne!(
        base,
        fingerprint(&[
            claim("t", "Revenue grew 21%"),
            evidence("t", "Revenue grew 12%")
        ])
    );
    let mut moved = evidence("t", "Revenue grew 12%");
    if let KnowledgeChange::AddProvenance {
        source_revision, ..
    } = &mut moved
    {
        *source_revision = "rev-2".into();
    }
    assert_ne!(base, fingerprint(&[claim("t", "Revenue grew 12%"), moved]));
}

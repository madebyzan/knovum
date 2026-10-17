use serde_json::json;

use super::{node_ids, records, require_single_primary, SourceInput};
use crate::spaces::derivation::inputs::RevisionBasis;

fn input(node_id: &str, role: &str, basis: RevisionBasis) -> SourceInput {
    SourceInput {
        node_id: node_id.into(),
        role: role.into(),
        source_revision: "sha256:source".into(),
        revision_basis: basis,
    }
}

#[test]
fn an_unindexed_image_is_pinned_on_its_asset() {
    let decoded: SourceInput = serde_json::from_value(json!({
        "nodeId": "image-1",
        "role": "primary",
        "sourceRevision": "sha256:pixels",
        "revisionBasis": "asset",
    }))
    .unwrap();
    assert_eq!(decoded.revision_basis, RevisionBasis::Asset);
    assert_eq!(
        records(&[decoded])[0].revision_basis,
        RevisionBasis::Asset,
        "the recorded basis must be the one the Run resolved"
    );
}

#[test]
fn sources_need_exactly_one_leading_primary() {
    assert!(require_single_primary(&[]).is_err());
    assert!(require_single_primary(&[input("a", "source", RevisionBasis::Knowledge)]).is_err());
    assert!(require_single_primary(&[
        input("a", "source", RevisionBasis::Knowledge),
        input("b", "primary", RevisionBasis::Knowledge),
    ])
    .is_err());
    assert!(require_single_primary(&[
        input("a", "primary", RevisionBasis::Knowledge),
        input("b", "primary", RevisionBasis::Knowledge),
    ])
    .is_err());
    assert!(require_single_primary(&[
        input("a", "primary", RevisionBasis::Knowledge),
        input("b", "source", RevisionBasis::Asset),
    ])
    .is_ok());
}

#[test]
fn node_ids_keep_the_selection_order() {
    assert_eq!(
        node_ids(&[
            input("first", "primary", RevisionBasis::Knowledge),
            input("second", "source", RevisionBasis::Asset),
        ]),
        vec!["first".to_owned(), "second".to_owned()]
    );
}

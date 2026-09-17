use super::*;

#[test]
fn delete_predicate_scopes_space_and_lists_all_node_ids() {
    let predicate =
        build_delete_node_predicate("space-a", &["n1".into(), "n2".into(), "n3".into()]);
    assert_eq!(
        predicate,
        "space_id = 'space-a' AND node_id IN ('n1','n2','n3')"
    );
}

#[test]
fn delete_predicate_escapes_single_quotes() {
    let predicate = build_delete_node_predicate("sp'ace", &["it's".into()]);
    assert_eq!(predicate, "space_id = 'sp''ace' AND node_id IN ('it''s')");
}

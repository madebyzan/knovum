use crate::spaces::knowledge::contracts::is_symmetric_relation_kind;

#[test]
fn only_semantically_symmetric_relations_are_bidirectional() {
    assert!(is_symmetric_relation_kind("related"));
    assert!(is_symmetric_relation_kind("contradicts"));
    for kind in [
        "reference",
        "supports",
        "depends-on",
        "supersedes",
        "answers",
        "uses",
        "derives-from",
        "caused-by",
    ] {
        assert!(
            !is_symmetric_relation_kind(kind),
            "{kind} must remain directional"
        );
    }
}

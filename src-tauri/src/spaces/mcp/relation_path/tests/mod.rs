use crate::spaces::knowledge::relations::is_symmetric_kind;

#[test]
fn only_semantically_symmetric_relations_are_bidirectional() {
    assert!(is_symmetric_kind("related"));
    assert!(is_symmetric_kind("contradicts"));
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
        assert!(!is_symmetric_kind(kind), "{kind} must remain directional");
    }
}

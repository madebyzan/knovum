use super::*;

#[test]
fn referenced_descendant_keeps_parent_namespace() {
    let referenced = BTreeSet::from(["/data/spaces/assets/web/favicon.png".to_owned()]);
    assert!(tree_is_referenced(
        Path::new("/data/spaces/assets/web"),
        &referenced,
    ));
    assert!(!tree_is_referenced(
        Path::new("/data/spaces/assets/orphan"),
        &referenced,
    ));
}

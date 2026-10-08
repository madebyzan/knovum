pub(crate) fn is_symmetric(kind: &str) -> bool {
    matches!(kind, "related" | "contradicts")
}

pub(crate) fn canonical(kind: &str, source_id: &str, target_id: &str) -> (String, String) {
    if is_symmetric(kind) && source_id > target_id {
        (target_id.to_owned(), source_id.to_owned())
    } else {
        (source_id.to_owned(), target_id.to_owned())
    }
}

use super::layer_title;

#[test]
fn layer_titles_follow_the_composite_order() {
    assert_eq!(layer_title("Sticker sheet", 0), "Sticker sheet-layer-1");
    assert_eq!(layer_title("Sticker sheet", 8), "Sticker sheet-layer-9");
}

#[test]
fn a_long_source_title_is_trimmed_to_fit_the_node_limit() {
    let source = "x".repeat(super::MAX_TITLE * 2);
    let title = layer_title(&source, 0);
    assert_eq!(title.encode_utf16().count(), super::MAX_TITLE);
    assert!(title.ends_with("-layer-1"));
}

#[test]
fn trimming_never_splits_a_character() {
    // Each emoji is two UTF-16 code units, so the budget cannot be filled exactly.
    let source = "🍌".repeat(super::MAX_TITLE);
    let title = layer_title(&source, 0);
    assert!(title.encode_utf16().count() <= super::MAX_TITLE);
    assert!(title.starts_with('🍌'));
    assert!(title.ends_with("-layer-1"));
}

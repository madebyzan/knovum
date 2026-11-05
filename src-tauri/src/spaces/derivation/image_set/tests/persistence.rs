use super::super::contracts::EXAMPLE as SET;
use super::{member_title, MAX_TITLE};

#[test]
fn member_titles_follow_the_publication_order() {
    assert_eq!(member_title(&SET, "Sheet", 0), "Sheet-part-1");
    assert_eq!(member_title(&SET, "Sheet", 8), "Sheet-part-9");
}

#[test]
fn a_long_source_title_is_trimmed_to_fit_the_node_limit() {
    let source = "x".repeat(MAX_TITLE * 2);
    let title = member_title(&SET, &source, 0);
    assert_eq!(title.encode_utf16().count(), MAX_TITLE);
    assert!(title.ends_with("-part-1"));
}

#[test]
fn trimming_never_splits_a_character() {
    // Each emoji is two UTF-16 code units, so the budget cannot be filled exactly.
    let source = "🍌".repeat(MAX_TITLE);
    let title = member_title(&SET, &source, 0);
    assert!(title.encode_utf16().count() <= MAX_TITLE);
    assert!(title.starts_with('🍌'));
    assert!(title.ends_with("-part-1"));
}

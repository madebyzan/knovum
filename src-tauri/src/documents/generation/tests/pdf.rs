use super::super::pdf;
use super::{fixture, source};

#[test]
fn typst_source_uses_structural_lists_tables_and_raw_code() {
    let rendered = pdf::tests::render_source(&fixture()).unwrap();
    for marker in [
        "#table(columns: 2",
        "#enum(start: 7",
        "#list(",
        "#raw(",
        "#link(",
    ] {
        assert!(rendered.contains(marker), "missing {marker}");
    }
    assert!(rendered.contains("#linebreak()"));
    assert!(!rendered.contains("lang: \"zh\""));
}

#[test]
fn typst_control_syntax_is_quoted_not_interpolated() {
    let mut input = source("`#read(\"private\")` and **bold**");
    input.title = "\"] #read(\"private\") [".into();
    let rendered = pdf::tests::render_source(&input).unwrap();
    assert!(rendered.contains("#text(\"\\\"] #read(\\\"private\\\") [\")"));
    assert!(rendered.contains("#raw(\"#read(\\\"private\\\")\")"));
}

#[test]
fn unsupported_binary_content_fails_before_typst_compilation() {
    assert!(pdf::tests::render_source(&source("[Link](file:///private/a)")).is_err());
}

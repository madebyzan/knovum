use super::super::{markdown, render_text, text};
use super::{fixture, source};

#[test]
fn plain_text_preserves_semantic_content_without_markdown_marks() {
    let bytes = render_text(&source(
        "Use **Rust** and [Reference](https://example.com).\n\n- [x] Done",
    ))
    .unwrap();
    let value = String::from_utf8(bytes).unwrap();
    assert!(value.contains("Use Rust and Reference (https://example.com)."));
    assert!(value.contains("- [x] Done"));
    assert!(!value.contains("**"));
}

#[test]
fn tables_and_code_keep_their_reading_order() {
    let value = text::render(&markdown::parse(&fixture()).unwrap());
    assert!(value.starts_with("中文 English document\n\n"));
    assert!(value.contains("Name\tValue\nRust\t42\n中文\tvalue\n"));
    assert!(value.contains("7. Ordered seven"));
    assert!(value.contains("8. Ordered eight"));
    assert!(value.contains("fn main() {\n    println!(\"Hello, 世界\");\n}"));
    assert!(value.contains("Hard break follows.\nNext line."));
}

#[test]
fn images_become_alt_text_without_exporting_private_paths() {
    let bytes = render_text(&source("![Diagram](/private/secret.png)")).unwrap();
    assert_eq!(String::from_utf8(bytes).unwrap(), "Diagram\n\n");
}

#[test]
fn title_only_documents_are_exportable() {
    let mut input = source("");
    input.title = "Title only".into();
    assert_eq!(render_text(&input).unwrap(), b"Title only\n\n");
}

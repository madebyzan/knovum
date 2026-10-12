use serde_json::{json, Value};

use super::source;
use crate::documents::generation::{input::note::parse, DocumentFormat};

#[test]
fn unknown_meaningful_nodes_and_marks_are_rejected() {
    for content in [
        json!({"type":"doc","content":[{"type":"video","attrs":{"src":"private"}}]}),
        json!({"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"x","marks":[{"type":"reasoningTrace"}]}]}]}),
    ] {
        let error = parse(&source(content), DocumentFormat::Docx).unwrap_err();
        assert!(error.contains("Unsupported Tiptap"));
    }
}

#[test]
fn text_depth_and_table_column_budgets_are_enforced() {
    let oversized = json!({"type":"doc","content":[{"type":"paragraph","content":[{
        "type":"text","text":"x".repeat(4 * 1024 * 1024 + 1)
    }]}]});
    assert!(parse(&source(oversized), DocumentFormat::Docx)
        .unwrap_err()
        .contains("4 MiB"));

    let mut nested = json!({"type":"paragraph"});
    for _ in 0..65 {
        nested = json!({"type":"blockquote","content":[nested]});
    }
    assert!(parse(
        &source(json!({"type":"doc","content":[nested]})),
        DocumentFormat::Docx
    )
    .unwrap_err()
    .contains("64 levels"));

    let cells: Vec<Value> = (0..65)
        .map(|_| {
            json!({
                "type":"tableCell","attrs":{"colspan":1,"rowspan":1,"colwidth":null,"align":null},
                "content":[{"type":"paragraph"}]
            })
        })
        .collect();
    let table = json!({"type":"doc","content":[{"type":"table","content":[{"type":"tableRow","content":cells}]}]});
    assert!(parse(&source(table), DocumentFormat::Docx)
        .unwrap_err()
        .contains("64 columns"));
}

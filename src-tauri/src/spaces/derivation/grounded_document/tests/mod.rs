use serde_json::json;

use super::prompt::request;

#[test]
fn a_document_with_sources_is_held_to_them_and_cites_them() {
    let input = request(
        "Brief",
        &[json!({"nodeTitle": "Spec", "text": "Step one."})],
    )
    .unwrap();
    assert!(input.system_prompt.contains("[C1]"));
    assert!(input.text.contains("C1 | Spec\nStep one."));
}

#[test]
fn a_document_with_nothing_selected_is_written_from_its_purpose_and_cites_nothing() {
    let input = request("Launch plan", &[]).unwrap();
    assert!(input.system_prompt.contains("Do not cite sources"));
    assert!(input.instruction.ends_with("Launch plan"));
    assert!(input.text.is_empty());
}

use super::prompt::assemble;

#[test]
fn prompt_keeps_source_knowledge_and_direction_separate() {
    let prompt = assemble("Scene one: verified fact.", "Slow camera movement").expect("prompt");
    assert!(prompt.contains("SOURCE KNOWLEDGE\nScene one: verified fact."));
    assert!(prompt.contains("CREATIVE DIRECTION\nSlow camera movement"));
}

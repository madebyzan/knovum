use super::prompt::assemble;

#[test]
fn prompt_keeps_grounded_script_and_direction_separate() {
    let prompt = assemble("Scene one: verified fact.", "Slow camera movement").expect("prompt");
    assert!(prompt.contains("SCRIPT\nScene one: verified fact."));
    assert!(prompt.contains("CREATIVE DIRECTION\nSlow camera movement"));
}

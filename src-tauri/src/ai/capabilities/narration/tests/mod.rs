use super::*;
use crate::ai::tasks::TaskTargetSelection;

fn request(mode: NarrationPreparationMode, text: &str) -> NarrationPreparationCommandRequest {
    NarrationPreparationCommandRequest {
        target: TaskTargetSelection::Model {
            connection_id: "conn-1".into(),
            model_id: "model-1".into(),
            temperature: 0.2,
        },
        mode,
        text: text.into(),
    }
}

#[test]
fn rejects_empty_and_oversized_narration_inputs() {
    assert!(input(&request(NarrationPreparationMode::Clean, "   \n  ")).is_err());
    let oversized = "a".repeat(MAX_NARRATION_INPUT_CHARS + 1);
    assert!(input(&request(NarrationPreparationMode::Adapt, &oversized)).is_err());
}

#[test]
fn builds_distinct_clean_and_adapt_prompts_without_markdown_instructions() {
    let clean = input(&request(
        NarrationPreparationMode::Clean,
        "  Revenue grew +12% YoY [C1].  ",
    ))
    .unwrap();
    let adapt = input(&request(
        NarrationPreparationMode::Adapt,
        "- Q3 revenue +12%\n- Costs flat",
    ))
    .unwrap();

    assert_eq!(clean.text, "Revenue grew +12% YoY [C1].");
    assert!(clean.system_prompt.contains("faithfully as possible"));
    assert!(clean.system_prompt.contains("NEVER output Markdown"));
    assert!(adapt.system_prompt.contains("spoken paragraphs"));
    assert!(adapt
        .system_prompt
        .contains("Do NOT add generic podcast intros"));
    assert_ne!(clean.system_prompt, adapt.system_prompt);
}

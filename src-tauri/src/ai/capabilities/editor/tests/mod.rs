use super::*;
use crate::ai::tasks::{text::EditorContextReference, TaskTargetSelection};

fn base_request(action: EditorTransformAction, text: &str) -> EditorTransformCommandRequest {
    EditorTransformCommandRequest {
        target: TaskTargetSelection::Model {
            connection_id: "conn".into(),
            model_id: "model".into(),
            temperature: 0.7,
        },
        action,
        text: text.into(),
        instruction: None,
        context: None,
        previous_draft: None,
    }
}

#[test]
fn custom_action_allows_empty_selection_when_instruction_and_context_exist() {
    let mut request = base_request(EditorTransformAction::Custom, "");
    request.instruction = Some("Draft a 3-row comparison table".into());
    request.context = Some(EditorDocumentContext {
        document_title: Some("Architecture Plan".into()),
        section_heading: Some("Storage > Options".into()),
        preceding_text: Some("We compared SQLite and LanceDB.".into()),
        following_text: None,
        references: Some(vec![EditorContextReference {
            node_id: "n1".into(),
            title: "Benchmark Notes".into(),
            excerpt: "SQLite latency is under 1ms.".into(),
        }]),
    });

    let built = input(&request, EditorMediaCapabilities::default()).expect("valid custom request");
    assert_eq!(built.instruction, "Draft a 3-row comparison table");
    assert!(built.text.contains("Document title: Architecture Plan"));
    assert!(built.text.contains("Current section: Storage > Options"));
    assert!(built.text.contains("[Benchmark Notes]"));
}

#[test]
fn complete_accepts_preceding_context_when_selection_is_empty() {
    let mut request = base_request(EditorTransformAction::Complete, "");
    request.context = Some(EditorDocumentContext {
        preceding_text: Some("The migration runs in three phases.".into()),
        ..Default::default()
    });
    let built = input(&request, EditorMediaCapabilities::default()).expect("valid continuation");
    assert!(built.text.contains("Preceding context:"));
}

#[test]
fn rejects_empty_custom_instruction_and_excess_references() {
    let empty = base_request(EditorTransformAction::Custom, "hello");
    assert!(input(&empty, EditorMediaCapabilities::default()).is_err());

    let mut too_many = base_request(EditorTransformAction::Custom, "hello");
    too_many.instruction = Some("Summarize".into());
    too_many.context = Some(EditorDocumentContext {
        references: Some(
            (0..=MAX_REFERENCES)
                .map(|index| EditorContextReference {
                    node_id: format!("n{index}"),
                    title: "Ref".into(),
                    excerpt: "Text".into(),
                })
                .collect(),
        ),
        ..Default::default()
    });
    assert!(input(&too_many, EditorMediaCapabilities::default()).is_err());
}

#[test]
fn follow_up_refinement_embeds_previous_draft() {
    let mut request = base_request(EditorTransformAction::Custom, "Original sentence.");
    request.instruction = Some("Make it half as long".into());
    request.previous_draft = Some("A slightly shorter draft sentence.".into());

    let built = input(
        &request,
        EditorMediaCapabilities {
            image: true,
            video: true,
        },
    )
    .expect("valid follow-up");
    assert!(built.system_prompt.contains("knovum-gen://image"));
    assert!(built.system_prompt.contains("knovum-gen://video"));
    assert!(built.instruction.contains("Refine the previous draft"));
    assert!(built
        .text
        .contains("Previous draft:\nA slightly shorter draft sentence."));
}

use std::collections::BTreeSet;

use crate::{agent_runtime::*, ai::contracts::*};

fn image(source: ModelBinarySource) -> ModelContentPart {
    ModelContentPart::Binary {
        media_type: "image/png".into(),
        source,
        name: Some("source.png".into()),
    }
}

fn state(content: Vec<ModelContentPart>) -> ControlCheckpoint {
    start(
        ModelMessage {
            role: ModelRole::User,
            content,
        },
        5,
        10,
        "run".into(),
        "segment".into(),
        "catalog".into(),
    )
    .unwrap()
}

fn messages(checkpoint: &mut ControlCheckpoint) -> Vec<ModelMessage> {
    match next(checkpoint).unwrap() {
        ControlDirective::CallModel { messages, .. } => messages,
        _ => panic!("expected model turn"),
    }
}

#[test]
fn images_survive_checkpoint_and_runtime_conversion_for_inline_and_uploaded_sources() {
    for source in [
        ModelBinarySource::Base64 {
            data: "aW1hZ2U=".into(),
        },
        ModelBinarySource::Url {
            url: "https://media.invalid/files/image".into(),
        },
    ] {
        let input = image(source);
        let mut checkpoint = state(vec![
            ModelContentPart::Text {
                text: "Inspect".into(),
            },
            input.clone(),
        ]);
        let encoded = serde_json::to_string(&checkpoint).unwrap();
        checkpoint = serde_json::from_str(&encoded).unwrap();
        assert!(requires_vision(&checkpoint).unwrap());
        let output = messages(&mut checkpoint);
        assert_eq!(
            serde_json::to_value(&output[0].content[1]).unwrap(),
            serde_json::to_value(input).unwrap()
        );
    }
}

#[test]
fn tool_read_images_follow_all_tool_receipts_and_remain_available_on_the_next_turn() {
    let mut checkpoint = state(vec![ModelContentPart::Text {
        text: "Read two images".into(),
    }]);
    messages(&mut checkpoint);
    accept_model(
        &mut checkpoint,
        ModelTurnResponse {
            message: ModelMessage {
                role: ModelRole::Assistant,
                content: ["a", "b"]
                    .into_iter()
                    .map(|id| ModelContentPart::ToolCall {
                        call_id: id.into(),
                        name: "read_node".into(),
                        arguments: serde_json::json!({}),
                        thought_signatures: None,
                    })
                    .collect(),
            },
            finish_reason: ModelFinishReason::ToolCall,
            finish_reason_raw: None,
            usage: None,
            provider: "test".into(),
            model: "test".into(),
            response_id: None,
        },
        BTreeSet::from(["read_node".into()]),
    )
    .unwrap();
    assert!(matches!(
        next(&mut checkpoint).unwrap(),
        ControlDirective::CallTools { .. }
    ));
    accept_tools(
        &mut checkpoint,
        ["a", "b"]
            .into_iter()
            .map(|id| RuntimeToolReply {
                call_id: id.into(),
                name: "read_node".into(),
                result: serde_json::json!({"nodeId":id}),
                media: vec![image(ModelBinarySource::Base64 {
                    data: "aW1hZ2U=".into(),
                })],
            })
            .collect(),
    )
    .unwrap();
    let encoded = serde_json::to_string(&checkpoint).unwrap();
    checkpoint = serde_json::from_str(&encoded).unwrap();
    assert!(requires_vision(&checkpoint).unwrap());
    let output = messages(&mut checkpoint);
    let tool = output
        .iter()
        .position(|message| matches!(message.role, ModelRole::Tool))
        .unwrap();
    assert_eq!(output[tool].content.len(), 2);
    assert!(output[tool]
        .content
        .iter()
        .all(|part| matches!(part, ModelContentPart::ToolResult { .. })));
    assert!(matches!(output[tool + 1].role, ModelRole::User));
    assert_eq!(
        output[tool + 1]
            .content
            .iter()
            .filter(|part| matches!(part, ModelContentPart::Binary { .. }))
            .count(),
        2
    );
}

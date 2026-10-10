use super::ImageOperation;

#[test]
fn prompt_removal_rejects_missing_and_oversized_descriptions() {
    for prompt in ["  ".to_owned(), "x".repeat(1001)] {
        assert!(ImageOperation::RemoveObjectPrompt { prompt }
            .validate()
            .is_err());
    }
    assert!(ImageOperation::RemoveObjectPrompt {
        prompt: "person in the background".into(),
    }
    .validate()
    .is_ok());
}

#[test]
fn normalized_selection_prompts_have_a_fixed_budget() {
    use super::ImageSelection;
    use crate::ai::contracts::image_compute::{ImageSelectionPoint, ImageSelectionRect};
    let mut selection = ImageSelection {
        points: vec![ImageSelectionPoint {
            x: 0.5,
            y: 0.5,
            foreground: true,
        }],
        box_rect: None,
        strokes: vec![],
    };
    assert!(selection.validate().is_ok());
    selection.points[0].x = f32::NAN;
    assert!(selection.validate().is_err());
    selection.points = vec![
        ImageSelectionPoint {
            x: 0.5,
            y: 0.5,
            foreground: true
        };
        5
    ];
    selection.box_rect = Some(ImageSelectionRect {
        x: 0.1,
        y: 0.1,
        width: 0.8,
        height: 0.8,
    });
    assert!(selection.validate().is_err());
    selection.points.clear();
    assert!(selection.validate().is_ok());
}

#[test]
fn local_operations_reject_provider_and_agent_execution_targets() {
    use super::ImageExecutionTarget;
    use crate::ai::{contracts::ExternalAgentKind, tasks::TaskTargetRef};
    for target in [
        TaskTargetRef::Model {
            connection_id: "connection".into(),
            model_id: "model".into(),
        },
        TaskTargetRef::Agent {
            runtime_kind: ExternalAgentKind::QoderCli,
        },
    ] {
        let execution = ImageExecutionTarget::Remote { target };
        for operation in [
            ImageOperation::Upscale {
                target_long_edge: 2048,
            },
            ImageOperation::RemoveObject { strokes: vec![] },
            ImageOperation::RemoveBackground,
            ImageOperation::GenerateDepth,
            ImageOperation::ExtractSelection {
                selection: super::ImageSelection {
                    points: vec![],
                    box_rect: None,
                    strokes: vec![],
                },
            },
        ] {
            assert!(execution
                .validate(&operation)
                .unwrap_err()
                .contains("requires a local model"));
        }
        assert!(execution
            .validate(&ImageOperation::RemoveObjectPrompt {
                prompt: "background person".into()
            })
            .is_ok());
    }
}

use serde_json::json;

use super::{definition, BuiltInWorkflowTemplateId};
use crate::spaces::workflows::{compile, definition::WorkflowNode};

#[test]
fn production_recipes_compile_with_review_gates() {
    let recipes = [
        (
            BuiltInWorkflowTemplateId::PresentationProduction,
            json!({"document": {"title": "Briefing", "purpose": "Create an executive briefing"}}),
        ),
        (
            BuiltInWorkflowTemplateId::VideoProduction,
            json!({
                "script": {"title": "Script", "prompt": "Explain the selected sources"},
                "video": {"title": "Video", "prompt": "Explain the selected sources", "aspectRatio": "16:9", "durationSeconds": 10}
            }),
        ),
        (
            BuiltInWorkflowTemplateId::SocialContentPackage,
            json!({
                "copy": {"title": "Post", "platform": "linkedin", "audience": "Product leaders", "goal": "Teach", "tone": "professional"},
                "image": {"title": "Visual", "imageStyle": "Editorial", "aspectRatio": "1:1"}
            }),
        ),
        (
            BuiltInWorkflowTemplateId::SubtitledVideoProduction,
            json!({}),
        ),
        (
            BuiltInWorkflowTemplateId::MeetingContentPackage,
            json!({"summary": {"title": "Summary", "summaryPurpose": "Capture decisions and owners"}}),
        ),
    ];

    for (id, parameters) in recipes {
        let recipe = definition(id, &parameters).expect("recipe definition");
        assert!(recipe
            .nodes
            .iter()
            .any(|node| matches!(node, WorkflowNode::Review { .. })));
        compile(&recipe).expect("built-in recipe must compile");
    }
}

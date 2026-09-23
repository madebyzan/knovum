use serde_json::{json, Value};

use crate::spaces::workflows::WorkflowDefinition;

mod building;

use building::{action, base, edges, input, join, output, review};

pub(super) struct ReviewedPair<'a> {
    pub title: &'a str,
    pub draft: (&'a str, &'a str, Value),
    pub review_purpose: &'a str,
    pub output: (&'a str, &'a str, Value),
}

pub(super) fn reviewed_pair(recipe: ReviewedPair<'_>) -> Result<WorkflowDefinition, String> {
    let (draft_id, draft_action, draft_parameters) = recipe.draft;
    let (output_id, output_action, output_parameters) = recipe.output;
    Ok(base(
        recipe.title,
        vec![
            input(),
            action(draft_id, draft_action, draft_parameters)?,
            review("review", recipe.review_purpose),
            join("approved"),
            action(output_id, output_action, output_parameters)?,
            output(),
        ],
        edges(&[
            ("input", draft_id),
            (draft_id, "review"),
            (draft_id, "approved"),
            ("review", "approved"),
            ("approved", output_id),
            (output_id, "output"),
        ]),
    ))
}

pub(super) fn social(parameters: &Value) -> Result<WorkflowDefinition, String> {
    Ok(base(
        "Social content package",
        vec![
            input(),
            action(
                "copy",
                "draft-social-post",
                super::take(parameters, "copy")?,
            )?,
            review(
                "review",
                "Review social copy before generating its companion image",
            ),
            join("approved"),
            action("image", "generate-image", super::take(parameters, "image")?)?,
            output(),
        ],
        edges(&[
            ("input", "copy"),
            ("copy", "review"),
            ("copy", "approved"),
            ("review", "approved"),
            ("approved", "image"),
            ("copy", "output"),
            ("image", "output"),
        ]),
    ))
}

pub(super) fn subtitled_video() -> Result<WorkflowDefinition, String> {
    Ok(base(
        "Subtitled video production",
        vec![
            input(),
            action("subtitles", "extract-subtitles", json!({}))?,
            review("review", "Review and edit subtitles before rendering"),
            join("approved"),
            action("video", "render-subtitled-video", json!({}))?,
            output(),
        ],
        edges(&[
            ("input", "subtitles"),
            ("subtitles", "review"),
            ("input", "approved"),
            ("subtitles", "approved"),
            ("review", "approved"),
            ("approved", "video"),
            ("video", "output"),
        ]),
    ))
}

pub(super) fn meeting(parameters: &Value) -> Result<WorkflowDefinition, String> {
    Ok(base(
        "Meeting content package",
        vec![
            input(),
            action("subtitles", "extract-subtitles", json!({}))?,
            action(
                "summary",
                "summarize-meeting",
                super::take(parameters, "summary")?,
            )?,
            review("review", "Review meeting subtitles before rendering"),
            join("media"),
            action("video", "render-subtitled-video", json!({}))?,
            output(),
        ],
        edges(&[
            ("input", "subtitles"),
            ("input", "summary"),
            ("subtitles", "review"),
            ("input", "media"),
            ("subtitles", "media"),
            ("review", "media"),
            ("media", "video"),
            ("summary", "output"),
            ("video", "output"),
        ]),
    ))
}

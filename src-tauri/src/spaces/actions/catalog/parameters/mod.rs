#![allow(dead_code)] // These structs are the source of generated JSON Schemas.

use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use serde_json::Value;
use std::sync::OnceLock;

pub(crate) mod media;

use media::ImageAspectRatio;
pub(super) use media::{image, video};

macro_rules! schema {
    ($name:ident, $type:ty) => {
        pub(super) fn $name() -> &'static Value {
            static SCHEMA: OnceLock<Value> = OnceLock::new();
            SCHEMA.get_or_init(|| {
                serde_json::to_value(schema_for!($type))
                    .expect("Space Action schema must serialize")
            })
        }
    };
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PurposeParameters {
    #[schemars(length(min = 1, max = 512))]
    title: String,
    #[schemars(length(min = 1))]
    purpose: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct NoteParameters {
    #[schemars(length(min = 1, max = 512))]
    title: String,
    #[schemars(length(min = 1, max = 4_194_304))]
    content: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct KnowledgeChangesParameters {
    operations: Vec<Value>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SpaceEditParameters {
    expected_edit_revision: u64,
    command: Value,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SearchParameters {
    #[schemars(length(min = 1))]
    query: String,
    #[schemars(range(min = 1, max = 100))]
    limit: u8,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct NodeParameters {
    #[schemars(length(min = 1))]
    node_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct QuestionParameters {
    #[schemars(length(min = 1))]
    question: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ScriptParameters {
    #[schemars(length(min = 1, max = 512))]
    title: String,
    #[schemars(length(min = 1))]
    prompt: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct MeetingSummaryParameters {
    #[schemars(length(min = 1, max = 512))]
    title: String,
    #[schemars(length(min = 1))]
    summary_purpose: String,
}

#[derive(Deserialize, JsonSchema, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum SocialPlatform {
    Generic,
    Xiaohongshu,
    Linkedin,
}

#[derive(Deserialize, JsonSchema, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum SocialTone {
    Professional,
    Friendly,
    Energetic,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SocialCopyParameters {
    #[schemars(length(min = 1, max = 512))]
    title: String,
    platform: SocialPlatform,
    #[schemars(length(min = 1))]
    audience: String,
    #[schemars(length(min = 1))]
    goal: String,
    tone: SocialTone,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct VisualizationParameters {
    #[schemars(length(min = 1, max = 512))]
    title: String,
    #[schemars(length(min = 1))]
    intent: String,
    #[schemars(length(min = 1))]
    style: String,
    aspect_ratio: ImageAspectRatio,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct EmptyParameters {}

schema!(purpose, PurposeParameters);
schema!(note, NoteParameters);
schema!(knowledge_changes, KnowledgeChangesParameters);
schema!(space_edit, SpaceEditParameters);
schema!(search, SearchParameters);
schema!(node, NodeParameters);
schema!(question, QuestionParameters);
schema!(script, ScriptParameters);
schema!(meeting_summary, MeetingSummaryParameters);
schema!(social_copy, SocialCopyParameters);
schema!(visualization, VisualizationParameters);
schema!(empty, EmptyParameters);

use serde::Deserialize;

use crate::ai::{
    contracts::{AgentKind, ExecutionCapability, ProviderKind},
    models::ExecutionTargetRef,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Envelope<T> {
    pub schema_version: u8,
    pub value: T,
}

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiSettings {
    pub(super) connections: Vec<Connection>,
    pub(super) desktop_agents: Vec<DesktopAgent>,
    pub(super) defaults: Defaults,
}

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename = "AiConnectionSettings")]
pub(crate) struct Connection {
    pub id: String,
    pub provider: ProviderKind,
    pub base_url: String,
    pub models: Vec<Model>,
    pub name: String,
}

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename = "AiModelDefinition")]
pub(crate) struct Model {
    pub id: String,
    pub model: String,
    pub capabilities: Vec<ExecutionCapability>,
    pub name: String,
}

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename = "DesktopAgentSettings")]
pub(crate) struct DesktopAgent {
    pub kind: AgentKind,
    pub enabled: bool,
}

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename = "AiRoutingDefaults")]
pub(crate) struct Defaults {
    pub writing: Option<ExecutionTargetRef>,
    pub knowledge: Option<ExecutionTargetRef>,
    pub visual_understanding: Option<ExecutionTargetRef>,
    pub translation: Option<ExecutionTargetRef>,
    pub image_generation: Option<ExecutionTargetRef>,
    pub video_generation: Option<ExecutionTargetRef>,
}

impl Defaults {
    pub(super) fn target(
        &self,
        purpose: crate::ai::contracts::RoutingPurpose,
    ) -> Option<ExecutionTargetRef> {
        use crate::ai::contracts::RoutingPurpose;
        match purpose {
            RoutingPurpose::Writing => self.writing.clone(),
            RoutingPurpose::Knowledge => self.knowledge.clone(),
            RoutingPurpose::VisualUnderstanding => self.visual_understanding.clone(),
            RoutingPurpose::Translation => self.translation.clone(),
            RoutingPurpose::ImageGeneration => self.image_generation.clone(),
            RoutingPurpose::VideoGeneration => self.video_generation.clone(),
        }
    }
}

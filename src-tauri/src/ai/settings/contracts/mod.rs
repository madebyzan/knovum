use serde::Deserialize;

use crate::ai::{
    contracts::{AgentKind, ExecutionCapability, ProviderKind},
    tasks::TaskTargetRef,
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
    #[ts(optional = nullable)]
    pub assistant: Option<AssistantModelSettings>,
}

#[derive(Clone, Deserialize, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantModelSettings {
    pub protocol: crate::ai::contracts::ChatModelProtocol,
    pub context_window: u32,
    pub max_output_tokens: u32,
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
    pub default_target: Option<TaskTargetRef>,
    pub writing: Option<TaskTargetRef>,
    pub knowledge: Option<TaskTargetRef>,
    pub visual_understanding: Option<TaskTargetRef>,
    pub translation: Option<TaskTargetRef>,
    pub image_generation: Option<TaskTargetRef>,
    pub video_generation: Option<TaskTargetRef>,
}

#[cfg(test)]
mod tests;

impl Defaults {
    pub(super) fn target(
        &self,
        purpose: crate::ai::contracts::RoutingPurpose,
    ) -> Option<TaskTargetRef> {
        use crate::ai::contracts::RoutingPurpose;
        let override_target = match purpose {
            RoutingPurpose::Writing => self.writing.clone(),
            RoutingPurpose::Knowledge => self.knowledge.clone(),
            RoutingPurpose::VisualUnderstanding => self.visual_understanding.clone(),
            RoutingPurpose::Translation => self.translation.clone(),
            RoutingPurpose::ImageGeneration => self.image_generation.clone(),
            RoutingPurpose::VideoGeneration => self.video_generation.clone(),
        };
        override_target.or_else(|| self.default_target.clone())
    }
}

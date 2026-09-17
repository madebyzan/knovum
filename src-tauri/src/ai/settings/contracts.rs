use serde::Deserialize;

use crate::ai::{
    contracts::{AgentKind, ExecutionCapability},
    models::ExecutionTargetRef,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Envelope<T> {
    pub schema_version: u8,
    pub value: T,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiSettings {
    pub(super) connections: Vec<Connection>,
    pub(super) desktop_agents: Vec<DesktopAgent>,
    pub(super) defaults: Defaults,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Connection {
    pub id: String,
    pub provider: String,
    pub base_url: String,
    pub models: Vec<Model>,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Model {
    pub id: String,
    pub model: String,
    pub capabilities: Vec<ExecutionCapability>,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct DesktopAgent {
    pub kind: AgentKind,
    pub enabled: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Defaults {
    pub writing: Option<ExecutionTargetRef>,
    pub knowledge: Option<ExecutionTargetRef>,
    pub visual_understanding: Option<ExecutionTargetRef>,
    pub translation: Option<ExecutionTargetRef>,
    pub image_generation: Option<ExecutionTargetRef>,
    pub video_generation: Option<ExecutionTargetRef>,
}

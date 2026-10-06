use super::{McpConnection, McpToolPolicy, SkillRevision};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentExtensionSelection {
    pub skill_ids: Vec<String>,
    pub connections: Vec<AgentConnectionSelection>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentConnectionSelection {
    pub connection_id: String,
    pub tool_names: Vec<String>,
    pub resource_uris: Vec<String>,
    pub resource_templates: Vec<McpResourceTemplateSelection>,
    pub prompt_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentSkillFile {
    pub path: String,
    pub encoding: SkillFileEncoding,
    pub content: String,
    pub digest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpResourceTemplateSelection {
    pub uri_template: String,
    pub parameters: std::collections::BTreeMap<String, McpResourceTemplateValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(untagged)]
pub enum McpResourceTemplateValue {
    String(String),
    List(Vec<String>),
    Map(std::collections::BTreeMap<String, String>),
    Undefined(()),
}

#[derive(Clone, Debug, Serialize, Deserialize, TS, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SkillFileEncoding {
    Utf8,
    Base64,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AgentToolBinding {
    SpaceAction {
        action_id: String,
        action_version: u16,
    },
    McpTool {
        connection_id: String,
        account_id: Option<String>,
        tool_name: String,
        schema_digest: String,
    },
    McpReadResource {
        connection_id: String,
        resource_uris: Vec<String>,
    },
    McpGetPrompt {
        connection_id: String,
        prompt_names: Vec<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentExtensionTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub binding: AgentToolBinding,
    pub read_only: bool,
    pub policy: McpToolPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentExtensionSnapshot {
    pub id: String,
    pub digest: String,
    pub skills: Vec<SkillRevision>,
    pub files: Vec<AgentSkillFile>,
    pub skill_tool_bindings: std::collections::BTreeMap<String, Vec<String>>,
    pub connections: Vec<McpConnection>,
    pub accounts: Vec<super::ConnectorAccount>,
    pub catalogs: std::collections::BTreeMap<String, super::McpConnectionCatalog>,
    pub tools: Vec<AgentExtensionTool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentExtensionAvailability {
    pub default_selection: AgentExtensionSelection,
    pub skills: Vec<SkillRevision>,
    pub connections: Vec<McpConnection>,
    pub diagnostics: Vec<String>,
}

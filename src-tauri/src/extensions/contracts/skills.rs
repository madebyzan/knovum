use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillRevision {
    pub id: String,
    pub plugin_id: Option<String>,
    pub name: String,
    pub description: String,
    pub digest: String,
    pub enabled: bool,
    pub requirements: Vec<String>,
    pub included_tools: Vec<String>,
    pub unsupported_requirements: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillContent {
    pub skill: SkillRevision,
    pub markdown: String,
    pub files: Vec<String>,
}

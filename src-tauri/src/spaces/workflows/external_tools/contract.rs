use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ExternalToolEffect {
    ReadOnly,
    Additive,
    Destructive,
}

impl ExternalToolEffect {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::Additive => "additive",
            Self::Destructive => "destructive",
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveExternalToolBindingRequest {
    pub id: String,
    pub space_id: String,
    pub name: String,
    pub endpoint: String,
    pub secret_key: Option<String>,
    pub tool_name: String,
    pub effect: ExternalToolEffect,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExternalToolBinding {
    pub id: String,
    pub space_id: String,
    pub name: String,
    pub endpoint: String,
    pub secret_key: Option<String>,
    #[ts(type = "Record<string, unknown>")]
    pub server_identity: serde_json::Value,
    pub tool_name: String,
    #[ts(type = "Record<string, unknown>")]
    pub input_schema: serde_json::Value,
    #[ts(type = "Record<string, unknown> | null")]
    pub output_schema: Option<serde_json::Value>,
    pub schema_fingerprint: String,
    pub effect: ExternalToolEffect,
    pub enabled: bool,
    pub reviewed_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum KnowledgeActorKind {
    Human,
    BuiltinAgent,
    ExternalAgent,
    System,
}

impl KnowledgeActorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::BuiltinAgent => "builtin_agent",
            Self::ExternalAgent => "external_agent",
            Self::System => "system",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        serde_json::from_value(serde_json::Value::String(value.to_owned()))
            .map_err(|_| "Unsupported knowledge actor kind".into())
    }
}

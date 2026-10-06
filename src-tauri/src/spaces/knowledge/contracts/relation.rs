use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum RelationKind {
    Reference,
    Related,
    Supports,
    Contradicts,
    DependsOn,
    Supersedes,
    Answers,
    Uses,
    DerivesFrom,
    CausedBy,
}

impl RelationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Related => "related",
            Self::Supports => "supports",
            Self::Contradicts => "contradicts",
            Self::DependsOn => "depends-on",
            Self::Supersedes => "supersedes",
            Self::Answers => "answers",
            Self::Uses => "uses",
            Self::DerivesFrom => "derives-from",
            Self::CausedBy => "caused-by",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        serde_json::from_value(serde_json::Value::String(value.to_owned()))
            .map_err(|_| "Unsupported Space relation kind".into())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum RelationStatus {
    Active,
    Contested,
    Invalidated,
}

impl RelationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Contested => "contested",
            Self::Invalidated => "invalidated",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        serde_json::from_value(serde_json::Value::String(value.to_owned()))
            .map_err(|_| "Unsupported Space relation status".into())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum RelationActorKind {
    Human,
    BuiltinAgent,
    ExternalAgent,
    System,
}

impl RelationActorKind {
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

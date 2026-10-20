use serde::{Deserialize, Serialize};

use crate::ai::{
    contracts::ExternalAgentKind,
    models::{ModelExecutionTarget, ModelTargetSelection},
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema, ts_rs::TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum TaskTarget {
    Model {
        connection_id: String,
        model_id: String,
    },
    Agent {
        runtime_kind: ExternalAgentKind,
    },
}

impl TaskTarget {
    pub fn with_temperature(self, temperature: f64) -> TaskTargetSelection {
        match self {
            Self::Model {
                connection_id,
                model_id,
            } => TaskTargetSelection::Model {
                connection_id,
                model_id,
                temperature,
            },
            Self::Agent { runtime_kind } => TaskTargetSelection::Agent { runtime_kind },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, ts_rs::TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum TaskTargetSelection {
    Model {
        connection_id: String,
        model_id: String,
        temperature: f64,
    },
    Agent {
        runtime_kind: ExternalAgentKind,
    },
}

impl From<ModelTargetSelection> for TaskTargetSelection {
    fn from(selection: ModelTargetSelection) -> Self {
        let ModelTargetSelection::Model {
            connection_id,
            model_id,
            temperature,
        } = selection;
        Self::Model {
            connection_id,
            model_id,
            temperature,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TaskExecutionTarget {
    Model(ModelExecutionTarget),
    Agent { runtime_kind: ExternalAgentKind },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ts_rs::TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ExecutionSource {
    Model { provider: String, model: String },
    Agent { runtime_kind: ExternalAgentKind },
}

impl ExecutionSource {
    pub fn engine(&self) -> String {
        match self {
            Self::Model { provider, model } => format!("{provider}:{model}"),
            Self::Agent { runtime_kind } => format!("desktop-agent:{}", runtime_kind.as_str()),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Model { provider, model } = self {
            if provider.trim().is_empty()
                || provider.encode_utf16().count() > 128
                || model.trim().is_empty()
                || model.encode_utf16().count() > 256
            {
                return Err("AI execution source has invalid model identity".into());
            }
        }
        Ok(())
    }
}

impl TaskExecutionTarget {
    pub fn source(&self) -> ExecutionSource {
        match self {
            Self::Model(target) => ExecutionSource::Model {
                provider: target.provider.clone(),
                model: target.model.clone(),
            },
            Self::Agent { runtime_kind } => ExecutionSource::Agent {
                runtime_kind: *runtime_kind,
            },
        }
    }
    pub fn engine(&self) -> String {
        self.source().engine()
    }
}

use serde::{Deserialize, Serialize};

use crate::ai::contracts::{AgentKind, ProviderKind};
use crate::ai::error::{AiError, AiResult};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum ExecutionTargetRef {
    Model {
        connection_id: String,
        model_id: String,
    },
    Agent {
        kind: AgentKind,
    },
}

impl ExecutionTargetRef {
    pub fn with_temperature(self, temperature: f64) -> ExecutionTargetSelection {
        match self {
            Self::Model {
                connection_id,
                model_id,
            } => ExecutionTargetSelection::Model {
                connection_id,
                model_id,
                temperature,
            },
            Self::Agent { kind } => ExecutionTargetSelection::Agent { kind },
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ExecutionTargetSelection {
    Model {
        connection_id: String,
        model_id: String,
        temperature: f64,
    },
    Agent {
        kind: AgentKind,
    },
}

#[derive(Clone, Debug)]
pub enum ExecutionTarget {
    Model {
        provider: String,
        model: String,
        base_url: String,
        temperature: f64,
        secret_key: String,
    },
    Agent {
        kind: AgentKind,
    },
}

impl ExecutionTarget {
    pub fn model_provider_kind(&self) -> AiResult<ProviderKind> {
        match self {
            Self::Model { provider, .. } => ProviderKind::parse(provider).ok_or_else(|| {
                AiError::unsupported(format!("Unsupported AI provider: {provider}"))
            }),
            Self::Agent { .. } => Err(AiError::unsupported(
                "Desktop Agents are not model providers",
            )),
        }
    }

    pub fn agent_kind(&self) -> Option<AgentKind> {
        match self {
            Self::Agent { kind, .. } => Some(*kind),
            Self::Model { .. } => None,
        }
    }

    pub fn provider(&self) -> &str {
        match self {
            Self::Model { provider, .. } => provider,
            Self::Agent { kind, .. } => kind.as_str(),
        }
    }

    pub fn model(&self) -> &str {
        match self {
            Self::Model { model, .. } => model,
            Self::Agent { kind, .. } => kind.as_str(),
        }
    }

    pub fn temperature(&self) -> Option<f64> {
        match self {
            Self::Model { temperature, .. } => Some(*temperature),
            Self::Agent { .. } => None,
        }
    }
}

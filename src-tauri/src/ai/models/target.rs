use serde::{Deserialize, Serialize};

use crate::ai::contracts::ProviderKind;
use crate::ai::error::{AiError, AiResult};

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
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
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
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
}

#[derive(Clone, Debug)]
pub struct ExecutionTarget {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub temperature: f64,
    pub secret_key: String,
}

impl ExecutionTarget {
    pub fn model_provider_kind(&self) -> AiResult<ProviderKind> {
        ProviderKind::parse(&self.provider).ok_or_else(|| {
            AiError::unsupported(format!("Unsupported AI provider: {}", self.provider))
        })
    }

    pub fn provider(&self) -> &str {
        &self.provider
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn temperature(&self) -> Option<f64> {
        Some(self.temperature)
    }
}

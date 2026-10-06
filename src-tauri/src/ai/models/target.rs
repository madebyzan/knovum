use serde::{Deserialize, Serialize};

use crate::ai::contracts::ProviderKind;
use crate::app_error::{AppError, AppResult};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, ts_rs::TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ModelTargetSelection {
    Model {
        connection_id: String,
        model_id: String,
        temperature: f64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelExecutionTarget {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub temperature: f64,
    pub secret_key: String,
    pub capabilities: Vec<crate::ai::contracts::ExecutionCapability>,
}

impl ModelExecutionTarget {
    pub fn model_provider_kind(&self) -> AppResult<ProviderKind> {
        ProviderKind::parse(&self.provider).ok_or_else(|| {
            AppError::unsupported(format!("Unsupported AI provider: {}", self.provider))
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

    pub fn is_local(&self) -> bool {
        self.provider == ProviderKind::Ollama.as_str()
            && reqwest::Url::parse(&self.base_url)
                .ok()
                .and_then(|url| url.host_str().map(str::to_owned))
                .is_some_and(|host| {
                    host.eq_ignore_ascii_case("localhost")
                        || host
                            .parse::<std::net::IpAddr>()
                            .is_ok_and(|address| address.is_loopback())
                })
    }
}

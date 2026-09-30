use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

pub(crate) mod contracts;
mod validation;

pub use contracts::AiSettings;
use contracts::Envelope;

use super::{
    contracts::ExecutionCapability,
    error::{AiError, AiResult},
    models::{ModelExecutionTarget, ModelTargetSelection},
    tasks::{TaskExecutionTarget, TaskTargetSelection},
};

const SETTINGS_FILE: &str = "settings.json";
const AI_SETTINGS_KEY: &str = "ai.models";
const SETTINGS_SCHEMA_VERSION: u8 = 1;

#[tauri::command]
pub fn validate_model_settings(settings: AiSettings) -> AiResult<()> {
    validation::validate(&settings)
}

pub fn resolve<R: Runtime>(
    app: &AppHandle<R>,
    selection: ModelTargetSelection,
    capability: ExecutionCapability,
) -> AiResult<ModelExecutionTarget> {
    let ModelTargetSelection::Model { temperature, .. } = &selection;
    if !temperature.is_finite() || !(0.0..=2.0).contains(temperature) {
        return Err(AiError::invalid("AI temperature must be between 0 and 2"));
    }
    let settings = load(app)?;
    match selection {
        ModelTargetSelection::Model {
            connection_id,
            model_id,
            temperature,
        } => resolve_model(settings, &connection_id, &model_id, temperature, capability),
    }
}

pub fn default_selection<R: Runtime>(
    app: &AppHandle<R>,
    purpose: crate::ai::contracts::RoutingPurpose,
) -> AiResult<TaskTargetSelection> {
    load(app)?
        .defaults
        .target(purpose)
        .map(|target| target.with_temperature(0.2))
        .ok_or_else(|| {
            AiError::not_configured(format!(
                "No AI execution method is selected for {:?}",
                purpose
            ))
        })
}

pub fn resolve_task<R: Runtime>(
    app: &AppHandle<R>,
    selection: TaskTargetSelection,
    capability: ExecutionCapability,
) -> AiResult<TaskExecutionTarget> {
    resolve_task_settings(load(app)?, selection, capability)
}

pub(crate) fn resolve_task_settings(
    settings: AiSettings,
    selection: TaskTargetSelection,
    capability: ExecutionCapability,
) -> AiResult<TaskExecutionTarget> {
    match selection {
        TaskTargetSelection::Model {
            connection_id,
            model_id,
            temperature,
        } => {
            if !temperature.is_finite() || !(0.0..=2.0).contains(&temperature) {
                return Err(AiError::invalid("AI temperature must be between 0 and 2"));
            }
            resolve_model(settings, &connection_id, &model_id, temperature, capability)
                .map(TaskExecutionTarget::Model)
        }
        TaskTargetSelection::Agent { runtime_kind } => {
            ensure_enabled(&settings, runtime_kind)?;
            Ok(TaskExecutionTarget::Agent { runtime_kind })
        }
    }
}

pub fn ensure_agent_enabled<R: Runtime>(
    app: &AppHandle<R>,
    kind: crate::ai::contracts::AgentKind,
) -> AiResult<()> {
    ensure_enabled(&load(app)?, kind)
}

fn ensure_enabled(settings: &AiSettings, kind: crate::ai::contracts::AgentKind) -> AiResult<()> {
    settings
        .desktop_agents
        .iter()
        .any(|agent| agent.kind == kind && agent.enabled)
        .then_some(())
        .ok_or_else(|| {
            AiError::new(
                super::error::AiErrorCode::AgentUnavailable,
                "Desktop Agent is not enabled",
            )
        })
}

fn load<R: Runtime>(app: &AppHandle<R>) -> AiResult<AiSettings> {
    let store = app
        .store(SETTINGS_FILE)
        .map_err(|error| AiError::internal(error.to_string()))?;
    let raw = store
        .get(AI_SETTINGS_KEY)
        .ok_or_else(|| AiError::not_configured("AI execution settings are not configured"))?;
    let envelope: Envelope<AiSettings> = serde_json::from_value(raw)
        .map_err(|error| AiError::invalid(format!("AI execution settings are invalid: {error}")))?;
    if envelope.schema_version != SETTINGS_SCHEMA_VERSION {
        return Err(AiError::invalid(
            "AI execution settings schema is unsupported",
        ));
    }
    validation::validate(&envelope.value)?;
    Ok(envelope.value)
}

fn resolve_model(
    settings: AiSettings,
    connection_id: &str,
    model_id: &str,
    temperature: f64,
    capability: ExecutionCapability,
) -> AiResult<ModelExecutionTarget> {
    let connection = settings
        .connections
        .into_iter()
        .find(|connection| connection.id == connection_id)
        .ok_or_else(|| AiError::invalid("AI connection does not exist"))?;
    let model = connection
        .models
        .into_iter()
        .find(|model| model.id == model_id)
        .ok_or_else(|| AiError::invalid("AI model does not exist"))?;
    if !model.capabilities.contains(&capability) {
        return Err(AiError::capability_not_supported(
            connection.provider.as_str(),
            "requested action",
        ));
    }
    let provider = connection.provider;
    Ok(ModelExecutionTarget {
        provider: provider.as_str().to_owned(),
        model: model.model,
        base_url: connection.base_url,
        temperature,
        secret_key: format!("integrations.ai.connections.{connection_id}.api-key"),
    })
}

#[cfg(test)]
mod task_routing_tests {
    use super::*;
    use serde_json::json;
    fn configured() -> serde_json::Value {
        json!({
            "connections":[], "desktopAgents":[{"kind":"codex","enabled":true}],
            "defaults":{"defaultTarget":{"type":"agent","runtimeKind":"codex"},"writing":null,"knowledge":null,"visualUnderstanding":null,"translation":null,"imageGeneration":null,"videoGeneration":null}
        })
    }
    #[test]
    fn native_model_contract_does_not_accept_an_agent_identity() {
        assert!(serde_json::from_value::<ModelTargetSelection>(
            json!({"type":"agent","runtimeKind":"codex"})
        )
        .is_err());
    }
    #[test]
    fn explicit_override_precedes_the_default_execution_method() {
        let mut value = configured();
        value["defaults"]["knowledge"] = json!({"type":"agent","runtimeKind":"qoder_cli"});
        let settings: AiSettings = serde_json::from_value(value).unwrap();
        assert_eq!(
            settings
                .defaults
                .target(crate::ai::contracts::RoutingPurpose::Knowledge),
            Some(crate::ai::tasks::TaskTargetRef::Agent {
                runtime_kind: crate::ai::contracts::AgentKind::QoderCli
            })
        );
        assert_eq!(
            settings
                .defaults
                .target(crate::ai::contracts::RoutingPurpose::Writing),
            Some(crate::ai::tasks::TaskTargetRef::Agent {
                runtime_kind: crate::ai::contracts::AgentKind::Codex
            })
        );
    }
    #[test]
    fn disabled_agent_is_unavailable_and_model_native_capabilities_remain_enforced() {
        let mut value = configured();
        value["desktopAgents"] = json!([]);
        let error = resolve_task_settings(
            serde_json::from_value(value).unwrap(),
            TaskTargetSelection::Agent {
                runtime_kind: crate::ai::contracts::AgentKind::Codex,
            },
            ExecutionCapability::VisionUnderstanding,
        )
        .unwrap_err();
        assert_eq!(
            error.code,
            super::super::error::AiErrorCode::AgentUnavailable
        );
        let mut value = configured();
        value["connections"] = json!([{"id":"byok","name":"User","provider":"openai","baseUrl":"https://api.openai.com/v1","models":[{"id":"text","name":"Text","model":"text","capabilities":["text-generation"]}]}]);
        let selection = TaskTargetSelection::Model {
            connection_id: "byok".into(),
            model_id: "text".into(),
            temperature: 0.2,
        };
        let target = resolve_task_settings(
            serde_json::from_value(value.clone()).unwrap(),
            selection.clone(),
            ExecutionCapability::TextGeneration,
        )
        .unwrap();
        assert!(matches!(target, TaskExecutionTarget::Model(_)));
        let error = resolve_task_settings(
            serde_json::from_value(value).unwrap(),
            selection,
            ExecutionCapability::VisionUnderstanding,
        )
        .unwrap_err();
        assert_eq!(
            error.code,
            super::super::error::AiErrorCode::CapabilityNotSupported
        );
    }
}

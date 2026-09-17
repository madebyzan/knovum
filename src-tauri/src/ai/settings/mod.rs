use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

mod contracts;
mod validation;

pub use contracts::AiSettings;
use contracts::Envelope;

use super::{
    contracts::{ExecutionCapability, ProviderKind},
    error::{AiError, AiResult},
    models::{ExecutionTarget, ExecutionTargetSelection},
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
    selection: ExecutionTargetSelection,
    capability: ExecutionCapability,
) -> AiResult<ExecutionTarget> {
    if let ExecutionTargetSelection::Model { temperature, .. } = &selection {
        if !temperature.is_finite() || !(0.0..=2.0).contains(temperature) {
            return Err(AiError::invalid("AI temperature must be between 0 and 2"));
        }
    }
    let settings = load(app)?;
    match selection {
        ExecutionTargetSelection::Model {
            connection_id,
            model_id,
            temperature,
        } => resolve_model(settings, &connection_id, &model_id, temperature, capability),
        ExecutionTargetSelection::Agent { kind } => {
            ensure_enabled(&settings, kind)?;
            Ok(ExecutionTarget::Agent { kind })
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
        .ok_or_else(|| AiError::invalid("Desktop Agent is not enabled"))
}

fn load<R: Runtime>(app: &AppHandle<R>) -> AiResult<AiSettings> {
    let store = app
        .store(SETTINGS_FILE)
        .map_err(|error| AiError::internal(error.to_string()))?;
    let raw = store
        .get(AI_SETTINGS_KEY)
        .ok_or_else(|| AiError::invalid("AI model settings are not configured"))?;
    let envelope: Envelope<AiSettings> = serde_json::from_value(raw)
        .map_err(|error| AiError::invalid(format!("AI model settings are invalid: {error}")))?;
    if envelope.schema_version != SETTINGS_SCHEMA_VERSION {
        return Err(AiError::invalid("AI model settings schema is unsupported"));
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
) -> AiResult<ExecutionTarget> {
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
            &connection.provider,
            "requested action",
        ));
    }
    let provider = ProviderKind::parse(&connection.provider)
        .ok_or_else(|| AiError::unsupported("AI provider is not registered"))?;
    Ok(ExecutionTarget::Model {
        provider: provider.as_str().to_owned(),
        model: model.model,
        base_url: connection.base_url,
        temperature,
        secret_key: format!("integrations.ai.connections.{connection_id}.api-key"),
    })
}

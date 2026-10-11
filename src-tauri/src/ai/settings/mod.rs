use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

mod builtin_agent;
pub(crate) mod contracts;
pub(crate) mod creation_library;
mod validation;
pub(crate) use builtin_agent::resolve as resolve_builtin_agent;

pub use contracts::AiSettings;
use contracts::Envelope;

use super::{
    contracts::ExecutionCapability,
    models::{ModelExecutionTarget, ModelTargetSelection},
    tasks::{TaskExecutionTarget, TaskTargetSelection},
};
use crate::foundation::app_error::{AppError, AppResult};

const SETTINGS_FILE: &str = "settings.json";
const AI_SETTINGS_KEY: &str = "ai.models";
const SETTINGS_SCHEMA_VERSION: u8 = 1;

#[tauri::command]
pub fn validate_model_settings(settings: AiSettings) -> AppResult<()> {
    validation::validate(&settings)
}

pub fn resolve<R: Runtime>(
    app: &AppHandle<R>,
    selection: ModelTargetSelection,
    capability: ExecutionCapability,
) -> AppResult<ModelExecutionTarget> {
    let ModelTargetSelection::Model { temperature, .. } = &selection;
    if !temperature.is_finite() || !(0.0..=2.0).contains(temperature) {
        return Err(AppError::invalid("AI temperature must be between 0 and 2"));
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
) -> AppResult<TaskTargetSelection> {
    load(app)?
        .defaults
        .target(purpose)
        .map(|target| target.with_temperature(0.2))
        .ok_or_else(|| {
            AppError::not_configured(format!(
                "No AI execution method is selected for {:?}",
                purpose
            ))
        })
}

pub fn resolve_task<R: Runtime>(
    app: &AppHandle<R>,
    selection: TaskTargetSelection,
    capability: ExecutionCapability,
) -> AppResult<TaskExecutionTarget> {
    resolve_task_settings(load(app)?, selection, capability)
}

pub(crate) fn resolve_task_settings(
    settings: AiSettings,
    selection: TaskTargetSelection,
    capability: ExecutionCapability,
) -> AppResult<TaskExecutionTarget> {
    match selection {
        TaskTargetSelection::Model {
            connection_id,
            model_id,
            temperature,
        } => {
            if !temperature.is_finite() || !(0.0..=2.0).contains(&temperature) {
                return Err(AppError::invalid("AI temperature must be between 0 and 2"));
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
    kind: crate::ai::contracts::ExternalAgentKind,
) -> AppResult<()> {
    ensure_enabled(&load(app)?, kind)
}

fn ensure_enabled(
    settings: &AiSettings,
    kind: crate::ai::contracts::ExternalAgentKind,
) -> AppResult<()> {
    settings
        .desktop_agents
        .iter()
        .any(|agent| agent.kind == kind && agent.enabled)
        .then_some(())
        .ok_or_else(|| AppError::new("agent_unavailable", "Desktop Agent is not enabled"))
}

fn load<R: Runtime>(app: &AppHandle<R>) -> AppResult<AiSettings> {
    let store = app
        .store(SETTINGS_FILE)
        .map_err(|error| AppError::internal(error.to_string()))?;
    let raw = store
        .get(AI_SETTINGS_KEY)
        .ok_or_else(|| AppError::not_configured("AI execution settings are not configured"))?;
    let envelope: Envelope<AiSettings> = serde_json::from_value(raw).map_err(|error| {
        AppError::invalid(format!("AI execution settings are invalid: {error}"))
    })?;
    if envelope.schema_version != SETTINGS_SCHEMA_VERSION {
        return Err(AppError::invalid(
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
) -> AppResult<ModelExecutionTarget> {
    let connection = settings
        .connections
        .into_iter()
        .find(|connection| connection.id == connection_id)
        .ok_or_else(|| AppError::invalid("AI connection does not exist"))?;
    let model = connection
        .models
        .into_iter()
        .find(|model| model.id == model_id)
        .ok_or_else(|| AppError::invalid("AI model does not exist"))?;
    if !model.capabilities.contains(&capability) {
        return Err(AppError::capability_not_supported(
            connection.provider.as_str(),
            "requested action",
        ));
    }
    let provider = connection.provider;
    Ok(ModelExecutionTarget {
        provider: provider.as_str().to_owned(),
        model: model.model,
        capabilities: model.capabilities,
        base_url: connection.base_url,
        temperature,
        secret_key: format!("integrations.ai.connections.{connection_id}.api-key"),
    })
}

#[cfg(test)]
mod tests;

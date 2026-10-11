use crate::foundation::app_error::AppError;
use crate::{
    ai::{contracts, providers},
    external_agents,
};

#[tauri::command]
pub async fn detect_external_agent_installations(
) -> Result<Vec<contracts::ExternalAgentInstallation>, AppError> {
    external_agents::detect_external_agent_installations().await
}

#[tauri::command]
pub fn list_external_agent_descriptors() -> Vec<contracts::ExternalAgentDescriptor> {
    external_agents::registry::descriptors()
}

#[tauri::command]
pub fn list_provider_descriptors() -> Vec<providers::registry::ProviderDescriptor> {
    providers::registry::descriptors().to_vec()
}

use crate::{
    ai::{contracts, error::AiError, providers},
    external_agents,
};

#[tauri::command]
pub async fn detect_agent_installations() -> Result<Vec<contracts::AgentInstallation>, AiError> {
    external_agents::detect_agent_installations().await
}

#[tauri::command]
pub fn list_agent_descriptors() -> Vec<contracts::AgentDescriptor> {
    external_agents::registry::descriptors()
}

#[tauri::command]
pub fn list_provider_descriptors() -> Vec<providers::registry::ProviderDescriptor> {
    providers::registry::descriptors().to_vec()
}

use crate::ai::{contracts, error::AiError, providers};

#[tauri::command]
pub async fn detect_agent_installations() -> Result<Vec<contracts::AgentInstallation>, AiError> {
    providers::agents::detect_agent_installations().await
}

#[tauri::command]
pub fn list_agent_descriptors() -> Vec<contracts::AgentDescriptor> {
    providers::agents::registry::descriptors()
}

#[tauri::command]
pub fn list_provider_descriptors() -> Vec<providers::registry::ProviderDescriptor> {
    providers::registry::descriptors().to_vec()
}

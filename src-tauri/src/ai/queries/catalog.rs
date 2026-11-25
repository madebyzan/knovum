use std::time::Duration;

use serde::Deserialize;

use crate::foundation::app_error::AppError;
use crate::{
    ai::{contracts, providers},
    external_agents,
};

const OLLAMA_TAGS_URL: &str = "http://127.0.0.1:11434/api/tags";
const OLLAMA_PROBE_TIMEOUT: Duration = Duration::from_millis(1_500);
const MAX_OLLAMA_MODELS: usize = 20;

#[derive(Deserialize)]
struct OllamaTagsResponse {
    #[serde(default)]
    models: Vec<OllamaTagEntry>,
}

#[derive(Deserialize)]
struct OllamaTagEntry {
    name: String,
}

#[tauri::command]
pub async fn detect_external_agent_installations(
) -> Result<Vec<contracts::ExternalAgentInstallation>, AppError> {
    external_agents::detect_external_agent_installations().await
}

#[tauri::command]
pub async fn detect_local_ollama_models() -> Result<Vec<String>, AppError> {
    let Ok(client) = reqwest::Client::builder()
        .no_proxy()
        .timeout(OLLAMA_PROBE_TIMEOUT)
        .build()
    else {
        return Ok(Vec::new());
    };
    let Ok(response) = client.get(OLLAMA_TAGS_URL).send().await else {
        return Ok(Vec::new());
    };
    if !response.status().is_success() {
        return Ok(Vec::new());
    }
    let Ok(payload) = response.json::<OllamaTagsResponse>().await else {
        return Ok(Vec::new());
    };
    let mut names = Vec::new();
    for entry in payload.models {
        let trimmed = entry.name.trim();
        if !trimmed.is_empty() && !names.iter().any(|existing| existing == trimmed) {
            names.push(trimmed.to_owned());
            if names.len() >= MAX_OLLAMA_MODELS {
                break;
            }
        }
    }
    Ok(names)
}

#[tauri::command]
pub fn list_external_agent_descriptors() -> Vec<contracts::ExternalAgentDescriptor> {
    external_agents::registry::descriptors()
}

#[tauri::command]
pub fn list_provider_descriptors() -> Vec<providers::registry::ProviderDescriptor> {
    providers::registry::descriptors().to_vec()
}

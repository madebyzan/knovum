use crate::ai::{
    error::{AiError, AiErrorCode, AiResult},
    models::ExecutionTarget,
};

#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub provider: String,
    pub model: String,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub temperature: f64,
}

impl SecurityContext {
    pub async fn from_target(target: &ExecutionTarget) -> AiResult<Self> {
        let ExecutionTarget {
            provider,
            model,
            base_url,
            temperature,
            secret_key,
        } = target;
        let model = model.trim();
        if model.is_empty() {
            return Err(AiError::not_configured("AI model is not configured."));
        }
        let endpoint = normalize_endpoint(base_url)?;
        let api_key = resolve_secret(provider, secret_key).await?;
        Ok(Self {
            provider: provider.clone(),
            model: model.to_string(),
            endpoint,
            api_key,
            temperature: *temperature,
        })
    }

    pub fn require_api_key(&self) -> AiResult<&str> {
        self.api_key.as_deref().ok_or_else(|| {
            AiError::new(
                AiErrorCode::NotConfigured,
                format!("API key is missing for provider '{}'.", self.provider),
            )
        })
    }
}

fn normalize_endpoint(base_url: &str) -> AiResult<String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(AiError::not_configured(
            "AI provider base URL is not configured.",
        ));
    }
    Ok(trimmed.to_string())
}

async fn resolve_secret(provider: &str, secret_key: &str) -> AiResult<Option<String>> {
    let key_name = secret_key.trim();
    let is_local = provider == "ollama";
    if is_local && key_name.is_empty() {
        return Ok(None);
    }
    let secret = crate::secrets::read_secret(key_name)
        .await
        .map_err(|err| AiError::new(AiErrorCode::ProviderRequestFailed, err.to_string()))?;
    match secret.filter(|v| !v.trim().is_empty()) {
        Some(val) => Ok(Some(val)),
        None if is_local => Ok(None),
        None => Err(AiError::not_configured(format!(
            "API key is missing for provider '{provider}'."
        ))),
    }
}

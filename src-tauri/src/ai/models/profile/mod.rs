//! A model's token limits are a property of the model. Providers publish them and the models.dev
//! catalog collects them — the same source LangChain bundles as chat-model profiles — so Knovum
//! resolves them instead of asking the user to copy them from documentation.
//!
//! Limits follow LangChain's profile semantics: `max_input_tokens` is the advertised input limit
//! (the context window when no separate input limit is published) and `max_output_tokens` the
//! advertised output limit; the Agent harness reserves output and headroom itself. A model the
//! catalog does not list has unknown limits, which is a valid state: the harness then compacts
//! when the provider reports a context overflow.

mod catalog;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::ai::contracts::ProviderKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ModelProfile {
    pub max_input_tokens: Option<u32>,
    pub max_output_tokens: Option<u32>,
}

pub(crate) async fn resolve<R: Runtime>(
    app: &AppHandle<R>,
    provider: ProviderKind,
    model: &str,
) -> ModelProfile {
    match catalog::current(app).await {
        Ok(catalog) => catalog.profile(provider, model),
        Err(error) => {
            log::warn!("Model limits for {model} stay unknown: {error}");
            ModelProfile::default()
        }
    }
}

#[cfg(test)]
mod tests;

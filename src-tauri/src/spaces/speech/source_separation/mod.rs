pub(crate) mod commands;
mod contracts;
mod execution;
mod media;
pub(super) mod publication;
mod repository;

#[cfg(test)]
mod tests;

pub use contracts::{SpeechSourceSeparationPublication, StartSpeechSourceSeparationRequest};
pub(crate) use execution::execute;
pub(super) use media::cleanup_stale_workspaces;

pub(super) const WORKFLOW: &str = "speech_source_separation_v1";

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SeparationConfiguration {
    source_node_id: String,
    vocals_label: String,
    background_label: String,
    source: super::video_config::SpeechMediaSource,
    model: crate::local_resources::model_catalog::LocalModelId,
}

impl SeparationConfiguration {
    fn from_claim(claim: &crate::spaces::runs::AttemptClaim) -> Result<Self, String> {
        let config: Self = serde_json::from_value(claim.configuration.clone())
            .map_err(crate::spaces::db::json_error)?;
        config.model.separation_spec()?;
        Ok(config)
    }
}

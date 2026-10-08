pub(crate) mod commands;
mod contracts;
mod execution;
mod media;
mod repository;

#[cfg(test)]
mod tests;

pub use contracts::{
    ReviewSpeechSourceSeparationRequest, SpeechSourceSeparationCandidate,
    SpeechSourceSeparationReviewStatus, StartSpeechSourceSeparationRequest,
};
pub(crate) use execution::execute;
pub(super) use media::cleanup_stale_workspaces;

#[cfg(test)]
pub(crate) const MODEL_SPEC_ID: &str = "spleeter-2stems-fp16-cpu-v1";
pub(super) const WORKFLOW: &str = "speech_source_separation_v1";

pub(crate) fn review_warning() -> &'static str {
    "vocals_accompaniment_requires_full_audition"
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SeparationConfiguration {
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

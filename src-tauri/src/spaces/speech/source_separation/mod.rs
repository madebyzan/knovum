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

pub(crate) const MODEL_SPEC_ID: &str = "spleeter-2stems-fp16-cpu-v1";
pub(super) const WORKFLOW: &str = "speech_source_separation_v1";

pub(crate) fn review_warning() -> &'static str {
    "vocals_accompaniment_requires_full_audition"
}

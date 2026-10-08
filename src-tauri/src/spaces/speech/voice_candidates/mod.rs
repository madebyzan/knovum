mod artifacts;
pub(crate) mod commands;
mod contracts;
mod execution;
mod recovery;
mod repository;
mod source_snapshot;
#[cfg(test)]
#[path = "tests/source_snapshot.rs"]
mod source_snapshot_tests;
mod start;

pub use contracts::{
    CreateCandidateVoiceRequest, DraftVoiceCandidateRequest, VoiceCandidate, VoiceCandidateSource,
    VoiceCandidateStatus, VoiceModel,
};
use contracts::{ReferenceSourceSnapshot, VoiceCandidateStepConfiguration};
pub(crate) use execution::execute;
pub(crate) use recovery::cleanup_terminal_snapshot;
pub(crate) use recovery::reconcile_terminal_snapshots;
pub(crate) use repository::{load_record, select_candidate};

pub(crate) const MODEL_SPEC_ID: &str = crate::local_resources::model_catalog::VOXCPM2_FULL_SPEC_ID;
pub(crate) const Q8_MODEL_SPEC_ID: &str = crate::local_resources::model_catalog::VOXCPM2_Q8_SPEC_ID;
pub(crate) const WORKFLOW: &str = "speech_voice_candidate_v1";

fn require_uuid(value: &str, label: &str) -> Result<(), String> {
    let parsed = uuid::Uuid::parse_str(value).map_err(|_| format!("{label} must be a UUID"))?;
    if parsed.to_string() != value {
        return Err(format!("{label} must use canonical UUID form"));
    }
    Ok(())
}

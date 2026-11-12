mod artifacts;
pub(crate) mod commands;
mod contracts;
mod execution;
mod recovery;
mod repository;
mod source_snapshot;
mod start;
#[cfg(test)]
mod tests;

pub use contracts::{
    CreateCandidateVoiceRequest, DraftVoiceCandidateRequest, VoiceCandidate, VoiceCandidateSource,
    VoiceCandidateStatus, VoiceModel,
};
use contracts::{ReferenceSourceSnapshot, VoiceCandidateStepConfiguration};
pub(crate) use execution::execute;
pub(crate) use recovery::cleanup_terminal_snapshot;
pub(crate) use recovery::reconcile_terminal_snapshots;
pub(crate) use repository::{load_record, select_candidate};

pub(crate) const WORKFLOW: &str = "speech_voice_candidate_v1";

fn require_uuid(value: &str, label: &str) -> Result<(), String> {
    let parsed = uuid::Uuid::parse_str(value).map_err(|_| format!("{label} must be a UUID"))?;
    if parsed.to_string() != value {
        return Err(format!("{label} must use canonical UUID form"));
    }
    Ok(())
}

pub(crate) mod commands;
mod contracts;
mod execution;
mod provenance;
mod repository;
#[cfg(test)]
mod tests;

pub(crate) use contracts::{TranslateVideoDubbingRequest, VideoTranslation};
pub(crate) use execution::execute;
pub(crate) use provenance::{validate_saved_provenance, SaveContext};

pub(crate) const NODE_KIND: &str = "speech_video_translation_segment";
pub(crate) const WORKFLOW: &str = "speech_video_translation_v1";

pub(crate) fn is_run_step(node_kind: &str) -> bool {
    node_kind == NODE_KIND
}

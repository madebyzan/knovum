use crate::dictation_contracts::{DictationPhase, DictationSnapshot};
use crate::local_resources::preparation::contracts::{
    ResourcePreparationProgress, ResourcePreparationStage,
};

pub(super) fn update(
    mut current: DictationSnapshot,
    progress: ResourcePreparationProgress,
) -> DictationSnapshot {
    if matches!(progress.stage, ResourcePreparationStage::Ready) {
        current.phase = DictationPhase::Transcribing;
        current.resources = None;
    } else {
        current.phase = DictationPhase::PreparingResources;
        current.resources = Some(progress);
    }
    current
}

#[cfg(test)]
#[path = "tests/progress.rs"]
mod tests;

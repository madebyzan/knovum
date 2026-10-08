pub(crate) mod asset_publication;
pub(crate) mod audio_format;
mod book_assets;
pub(crate) mod book_import;
mod cancellation;
pub(crate) mod commands;
pub(crate) mod contracts;
pub(in crate::spaces) mod delivery_instruction;
pub(crate) mod execution;
mod generation_attempt;
mod media_source;
pub(crate) mod preset_voices;
pub(crate) mod project_config;
mod project_publication;
pub(crate) mod project_source;
mod publication;
pub(crate) mod reference_preparation;
mod repository;
pub(crate) mod revisions;
pub(crate) mod run_execution;
mod run_start;
pub(in crate::spaces) mod runtime_identity;
pub(crate) mod signal_quality;
pub(crate) mod source_separation;
pub(crate) mod subtitle_import;
#[cfg(test)]
mod tests;
pub(crate) mod text;
pub(crate) mod translation;
pub(crate) mod video_config;
pub(crate) mod video_delivery;
mod video_publication;
pub(crate) mod voice_assets;
pub(crate) mod voice_candidates;
pub(crate) mod voice_creation;
pub(crate) mod voice_management;
mod voice_reference_validation;

pub(crate) async fn reconcile_interrupted(app: &tauri::AppHandle) -> Result<(), String> {
    source_separation::cleanup_stale_workspaces(app)?;
    publication::reconcile_interrupted(app).await
}

pub(crate) mod asset_publication;
mod cancellation;
pub(crate) mod commands;
pub(crate) mod contracts;
pub(crate) mod execution;
mod generation_attempt;
pub(crate) mod project_config;
mod project_publication;
mod publication;
mod repository;
pub(crate) mod revisions;
pub(crate) mod run_execution;
mod run_start;
pub(crate) mod source_separation;
#[cfg(test)]
mod tests;
pub(crate) mod text;
pub(crate) mod translation;
pub(crate) mod video_config;
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

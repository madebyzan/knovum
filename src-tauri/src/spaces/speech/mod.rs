mod cancellation;
pub(crate) mod commands;
pub(crate) mod contracts;
pub(crate) mod execution;
mod generation_attempt;
mod project_publication;
mod publication;
mod repository;
pub(crate) mod revisions;
pub(crate) mod text;

pub(crate) async fn reconcile_interrupted(app: &tauri::AppHandle) -> Result<(), String> {
    publication::reconcile_interrupted(app).await
}

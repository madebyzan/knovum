pub(crate) mod commands;
mod contracts;
mod document_export;
mod failure;
mod grounded_document;
mod image_cutout;
pub(crate) mod inputs;
mod knowledge_video;
mod metadata_sources;
pub(crate) mod output_paths;
mod presentation_export;
pub(crate) mod queries;
mod relations;
mod source_validation;
mod subtitle_extraction;
mod subtitled_video;
mod visualization;
pub(crate) mod workflow_runs;
pub(crate) mod workflows;

use tauri::AppHandle;

pub use contracts::{CompletedDerivationOutput, DerivationRelation};

pub(crate) async fn reconcile_interrupted(app: &AppHandle) -> Result<(), String> {
    failure::reconcile_interrupted(app).await?;
    workflow_runs::reconcile_interrupted(app).await
}

pub(crate) fn stage(channel: &tauri::ipc::Channel<String>, value: &str) -> Result<(), String> {
    channel
        .send(value.to_owned())
        .map_err(|error| error.to_string())
}

pub(crate) async fn persist_output_node(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    node_id: &str,
    data: &serde_json::Value,
    now: i64,
) -> Result<(), String> {
    sqlx::query("UPDATE spaces_nodes SET data=$1,updated_at=$2 WHERE id=$3")
        .bind(serde_json::to_string(data).map_err(crate::spaces::db::json_error)?)
        .bind(now)
        .bind(node_id)
        .execute(&mut **tx)
        .await
        .map_err(crate::spaces::db::sql_error)?;
    Ok(())
}

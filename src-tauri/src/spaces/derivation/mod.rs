pub(crate) mod action_execution;
pub(crate) mod action_request;
pub(crate) mod commands;
mod contracts;
mod document_export;
mod failure;
mod grounded_answer;
pub(crate) mod grounded_document;
mod image_cutout;
pub(crate) mod image_layers;
pub(crate) mod image_operations;
pub(crate) mod input_revision;
pub(crate) mod inputs;
pub(crate) mod metadata_sources;
pub(crate) mod output_paths;
mod pending_output;
mod presentation_publication;
pub(crate) mod queries;
mod relations;
mod scratch_file;
mod source_snapshot;
mod source_validation;
mod subtitle_extraction;
mod subtitled_video;
mod video_edit;
pub(crate) mod video_generation;
mod video_output;
pub(crate) mod visualization;

use tauri::AppHandle;

pub(crate) use action_execution::execute as execute_action;
pub use contracts::{CompletedDerivationOutput, DerivationRelation};

pub(crate) async fn reconcile_interrupted(app: &AppHandle) -> Result<(), String> {
    failure::reconcile_interrupted(app).await
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

#[cfg(test)]
mod tests;

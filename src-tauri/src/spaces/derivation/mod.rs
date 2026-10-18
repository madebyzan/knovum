pub(crate) mod action_execution;
pub(crate) mod action_request;
pub(crate) mod commands;
mod contracts;
mod failure;
mod grounded_answer;
pub(crate) mod grounded_document;
pub(crate) mod image_layers;
pub(crate) mod image_operations;
mod image_tensors;
pub(crate) mod inheritance;
pub(crate) mod input_revision;
pub(crate) mod inputs;
pub(crate) mod metadata_sources;
pub(crate) mod output_paths;
mod pending_output;
pub(super) mod placement;
pub(crate) mod queries;
pub(super) mod relations;
pub(crate) mod source_inputs;
mod source_snapshot;
mod source_validation;
pub(crate) mod subtitled_video;
pub(crate) mod video_edit;
pub(crate) mod video_generation;
mod video_output;
pub(crate) mod visualization;

use tauri::AppHandle;

use crate::foundation::app_error::AppError;
pub(crate) use action_execution::{execute as execute_action, ExecutionAuthority};
pub use contracts::CompletedDerivationOutput;

pub(crate) async fn reconcile_interrupted(app: &AppHandle) -> Result<(), String> {
    failure::reconcile_interrupted(app).await
}

pub(crate) fn stage(channel: &tauri::ipc::Channel<String>, value: &str) -> Result<(), String> {
    channel
        .send(value.to_owned())
        .map_err(|error| error.to_string())
}

/// Settles a derivation's output node onto its content. A derivation publishes through here rather
/// than through a node mutation, so this is where an output node reaches the same standing as any
/// other: its canvas footprint follows its content, and the knowledge its new asset calls for is
/// scheduled. What that is follows from the payload itself — a carried-over analysis pinned to the
/// published asset asks for nothing, a dropped one asks for a fresh reading.
pub(crate) async fn persist_output_node(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    node_id: &str,
    payload: &serde_json::Value,
    now: i64,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), AppError> {
    sqlx::query("UPDATE spaces_nodes SET data=$1,updated_at=$2 WHERE id=$3")
        .bind(sqlx::types::Json(payload))
        .bind(now)
        .bind(node_id)
        .execute(&mut **tx)
        .await
        .map_err(crate::spaces::db::sql_error)?;
    let space_id: String = sqlx::query_scalar("SELECT space_id FROM spaces_nodes WHERE id=$1")
        .bind(node_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(crate::spaces::db::sql_error)?;
    crate::spaces::assets::enrichment::jobs::intent::synchronize_for_node(
        tx,
        &space_id,
        node_id,
        payload,
        now,
        enrichment_target,
    )
    .await?;
    if !matches!(payload["kind"].as_str(), Some("image" | "video" | "file")) {
        return Ok(());
    }
    crate::spaces::nodes::sync_content_sizes_in_transaction(tx, &space_id, &[node_id.to_owned()])
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests;

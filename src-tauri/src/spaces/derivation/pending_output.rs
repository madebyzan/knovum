//! Output nodes a derivation publishes into. A pending node is reserved before its content
//! exists, so the canvas shows the work in progress and the handler has a stable node identity to
//! settle against. Reservation is shared by every Action: those that publish exactly one output
//! have it reserved before the handler runs, while an Action whose input decides the count
//! reserves its outputs from the handler once discovery has reported how many there are.

use serde_json::Value;
use tauri::AppHandle;

use super::action_execution::ExecutionAuthority;
use crate::foundation::app_error::AppError;
use crate::spaces::{
    db,
    knowledge::{indexing::KnowledgeChunkInput, model::revision},
    nodes::{self, contracts::InsertNode},
    structure::{self, SizingMode, CONTENT_GAP},
};

/// Canvas footprint reserved for one pending output before its content reports a size.
const NODE_WIDTH: f64 = 320.0;
const NODE_HEIGHT: f64 = 220.0;

pub(super) struct PendingOutput {
    pub(super) node_id: String,
    pub(super) kind: &'static str,
    pub(super) data: Value,
}

/// Reserves pending output nodes beside their sources. Outputs reserved together are laid out as
/// one row so a split reads left to right, and the row is placed as a single block to keep it
/// clear of existing content.
pub(super) async fn reserve(
    app: &AppHandle,
    space_id: &str,
    outputs: Vec<PendingOutput>,
    sources: &[String],
    authority: &ExecutionAuthority,
) -> Result<(), AppError> {
    let span = u32::try_from(outputs.len()).map_err(|_| "Too many derivation outputs")?;
    let span = f64::from(span);
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    let (mut snapshot, base) =
        structure::repository::load_in_transaction(&mut tx, space_id).await?;
    let before = structure::history::capture(&mut tx, space_id, &snapshot).await?;
    let (x, y) = structure::place_new_content(
        &snapshot.structure,
        &snapshot.layouts,
        sources,
        span * NODE_WIDTH + (span - 1.0) * CONTENT_GAP,
        NODE_HEIGHT,
    )?;
    let mut inserts = Vec::with_capacity(outputs.len());
    for (index, mut output) in outputs.into_iter().enumerate() {
        revision::apply(&mut output.data)?;
        inserts.push(InsertNode {
            id: output.node_id,
            kind: output.kind.into(),
            initial_x: x + offset(index) * (NODE_WIDTH + CONTENT_GAP),
            initial_y: y,
            width: NODE_WIDTH,
            height: NODE_HEIGHT,
            sizing_mode: SizingMode::Content,
            appearance: None,
            file_subpath: None,
            data: output.data,
            source_chunks: Vec::<KnowledgeChunkInput>::new(),
        });
    }
    structure::creation::append_content(&mut snapshot, &inserts, None)?;
    nodes::insert_rows_with_actor(
        &mut tx,
        space_id,
        &inserts,
        crate::spaces::knowledge::events::EventActor::new("system", Some(&authority.run_id)),
        crate::foundation::clock::unix_time_millis()?,
        crate::spaces::assets::enrichment::jobs::intent::selection(app)?.as_ref(),
    )
    .await?;
    let revision =
        structure::repository::save_in_transaction(&mut tx, space_id, base, &snapshot).await?;
    let after = structure::history::capture(&mut tx, space_id, &snapshot).await?;
    structure::history::record(&mut tx, space_id, &before, &after, revision).await?;
    Ok(tx.commit().await.map_err(db::sql_error)?)
}

/// Row offset of one output. The count is bounded before the row is laid out, so every index fits.
fn offset(index: usize) -> f64 {
    f64::from(u32::try_from(index).unwrap_or_default())
}

pub(super) async fn exists(
    app: &AppHandle,
    space_id: &str,
    node_id: &str,
) -> Result<bool, AppError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_one(db::pool(app).await?)
    .await
    .map_err(db::sql_error)?;
    Ok(count == 1)
}

pub(super) fn validate_unpublished_asset(output: &Value) -> Result<&str, String> {
    let output_path = output
        .get("assetPath")
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty())
        .ok_or("Derived output path is unavailable")?;
    if output.get("processingState").and_then(Value::as_str) != Some("processing")
        || output.get("sourceRevision").is_some()
        || output.get("previewPath").is_some()
    {
        return Err("Derived output must be an unpublished asset".into());
    }
    Ok(output_path)
}

pub(super) fn validate_separate_asset(output: &Value, source_path: &str) -> Result<(), String> {
    if validate_unpublished_asset(output)? == source_path {
        return Err("Derived output must use a separate asset path".into());
    }
    Ok(())
}

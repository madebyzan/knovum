//! Output nodes a derivation publishes into. A pending node is reserved before its content
//! exists, so the canvas shows the work from the moment it is asked for and the handler has a
//! stable node identity to settle against. Every Action has its first output reserved before the
//! handler runs; an Action whose input decides the count reserves the rest from the handler, and
//! the whole set is then laid out together.

use serde_json::Value;
use sqlx::Sqlite;
use tauri::AppHandle;

use super::action_execution::ExecutionAuthority;
use super::placement::{self, DerivedOutputs, PLACEHOLDER};
use super::relations;
use crate::foundation::app_error::AppError;
use crate::spaces::{
    db,
    knowledge::{indexing::KnowledgeChunkInput, model::revision},
    nodes::{self, contracts::InsertNode},
    structure::{self, SizingMode},
};

pub(super) struct PendingOutput {
    pub(super) node_id: String,
    pub(super) kind: &'static str,
    pub(super) data: Value,
}

/// Reserves pending output nodes in their sources' derivation field, together with the lineage
/// that explains them: a derived node and its `derives-from` Relations are created in one
/// transaction, so the canvas never shows an output floating unconnected while its work runs. An
/// output that is already reserved is placed into the set rather than inserted a second time,
/// which is how an Action that reserved its first output up front settles the whole set; its
/// lineage was recorded when it was first reserved.
pub(super) async fn reserve(
    app: &AppHandle,
    space_id: &str,
    outputs: Vec<PendingOutput>,
    sources: &[String],
    lineage: &'static str,
    authority: &ExecutionAuthority,
) -> Result<(), AppError> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    let (mut snapshot, base) =
        structure::repository::load_in_transaction(&mut tx, space_id).await?;
    let before = structure::history::capture(&mut tx, space_id, &snapshot).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let reserved = reserved_outputs(&mut tx, space_id, &outputs).await?;
    let cells = placement::plan(
        &mut tx,
        space_id,
        &snapshot.structure,
        &snapshot.layouts,
        DerivedOutputs {
            sources,
            reserved: &reserved,
            sizes: outputs
                .iter()
                .map(|output| placement::fitted(&output.data, &PLACEHOLDER, SizingMode::Content))
                .collect(),
        },
    )
    .await?;
    let mut inserts = Vec::with_capacity(outputs.len());
    for (mut output, cell) in outputs.into_iter().zip(cells) {
        revision::apply(&mut output.data)?;
        if reserved.contains(&output.node_id) {
            sqlx::query(
                "UPDATE spaces_nodes SET data=$1,updated_at=$2 WHERE id=$3 AND space_id=$4",
            )
            .bind(sqlx::types::Json(&output.data))
            .bind(now)
            .bind(&output.node_id)
            .bind(space_id)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
            structure::creation::place_existing_content(
                &mut snapshot,
                &output.node_id,
                cell.x,
                cell.y,
                cell.size,
            )?;
            continue;
        }
        inserts.push(InsertNode {
            id: output.node_id,
            kind: output.kind.into(),
            initial_x: cell.x,
            initial_y: cell.y,
            width: cell.size.width,
            height: cell.size.height,
            sizing_mode: SizingMode::Content,
            appearance: None,
            file_subpath: None,
            data: output.data,
            source_chunks: Vec::<KnowledgeChunkInput>::new(),
        });
    }
    if !inserts.is_empty() {
        structure::creation::append_content(&mut snapshot, &inserts, None)?;
        nodes::insert_rows_with_actor(
            &mut tx,
            space_id,
            &inserts,
            crate::spaces::knowledge::events::EventActor::new("system", Some(&authority.run_id)),
            now,
            crate::spaces::assets::enrichment::jobs::intent::selection(app)?.as_ref(),
        )
        .await?;
        for insert in &inserts {
            for source in sources {
                relations::derives_from(&mut tx, space_id, &insert.id, source, lineage, now)
                    .await?;
            }
        }
    }
    let revision =
        structure::repository::save_in_transaction(&mut tx, space_id, base, &snapshot).await?;
    let after = structure::history::capture(&mut tx, space_id, &snapshot).await?;
    structure::history::record(&mut tx, space_id, &before, &after, revision).await?;
    Ok(tx.commit().await.map_err(db::sql_error)?)
}

/// Which of these outputs are already reserved, so they are placed into the set instead of
/// inserted again — and so they do not count as content blocking the set's own placement.
async fn reserved_outputs(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    space_id: &str,
    outputs: &[PendingOutput],
) -> Result<Vec<String>, AppError> {
    let mut reserved = Vec::new();
    for output in outputs {
        if exists(&mut **tx, space_id, &output.node_id).await? {
            reserved.push(output.node_id.clone());
        }
    }
    Ok(reserved)
}

pub(super) async fn exists<'e, E>(
    executor: E,
    space_id: &str,
    node_id: &str,
) -> Result<bool, AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_one(executor)
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

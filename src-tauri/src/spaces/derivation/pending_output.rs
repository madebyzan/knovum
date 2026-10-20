//! Output nodes a Run publishes into. A pending node is reserved before its content exists, so the
//! canvas shows the work from the moment it is asked for and the publisher has a stable node
//! identity to settle against. Every output a Space produces from existing nodes goes through here
//! — an Action's outputs, generated speech, separated tracks — so one rule covers all of them: the
//! node and the `derives-from` Relations that explain it are created in one transaction, and a
//! reservation never outlives the Run that made it.
//!
//! A reservation names where its output intends to publish. The file itself is named by whoever
//! writes it, which is also what settles the node, so a path reserved for a file that is never
//! written is simply forgotten.

use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use super::action_execution::ExecutionAuthority;
use super::placement::{self, DerivedCell, DerivedOutputs};
use super::relations;
use crate::foundation::app_error::AppError;
use crate::spaces::{
    db,
    knowledge::{indexing::KnowledgeChunkInput, model::revision},
    nodes::{self, contracts::InsertNode},
    structure::{self, Size, SizingMode},
};

/// What a Run is told when the node it owes content to has been removed by a person. Publishing
/// would resurrect it, and reporting success would claim a result nobody can see.
pub(crate) const OUTPUT_DELETED: &str = "Its output node was deleted";

/// Canvas footprint of content that cannot report a size until it is published.
pub(crate) const PLACEHOLDER: Size = placement::PLACEHOLDER;

pub(crate) struct PendingOutput {
    pub(crate) node_id: String,
    pub(crate) kind: &'static str,
    pub(crate) data: Value,
    /// Canvas footprint until the published content reports its own.
    pub(crate) footprint: Size,
}

/// Reserves pending output nodes in their sources' derivation field, together with the lineage that
/// explains them. An output that is already reserved is placed into the set rather than inserted a
/// second time, which is how a Run that reserved its first output up front settles the whole set;
/// its lineage was recorded when it was first reserved.
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
    reserve_in_transaction(
        &mut tx,
        space_id,
        outputs,
        sources,
        lineage,
        &authority.run_id,
        crate::spaces::assets::enrichment::jobs::intent::selection(app)?.as_ref(),
    )
    .await?;
    Ok(tx.commit().await.map_err(db::sql_error)?)
}

/// Reserves outputs inside the caller's transaction, which is how a Run that reserves its output
/// as it is created cannot exist without it.
pub(crate) async fn reserve_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    outputs: Vec<PendingOutput>,
    sources: &[String],
    lineage: &'static str,
    run_id: &str,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), AppError> {
    let (mut snapshot, base) = structure::repository::load_in_transaction(tx, space_id).await?;
    let before = structure::history::capture(tx, space_id, &snapshot).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let reserved = reserved_outputs(tx, space_id, &outputs).await?;
    let cells = placement::plan(
        tx,
        space_id,
        &snapshot.structure,
        &snapshot.layouts,
        DerivedOutputs {
            sources,
            reserved: &reserved,
            sizes: outputs
                .iter()
                .map(|output| {
                    placement::fitted(&output.data, &output.footprint, SizingMode::Content)
                })
                .collect(),
        },
    )
    .await?;
    let mut inserts = Vec::with_capacity(outputs.len());
    for (mut output, cell) in outputs.into_iter().zip(cells) {
        revision::apply(&mut output.data)?;
        if reserved.contains(&output.node_id) {
            replace_reserved(tx, space_id, &mut snapshot, &output, &cell, now).await?;
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
            tx,
            space_id,
            &inserts,
            crate::spaces::knowledge::events::EventActor::new("system", Some(run_id)),
            now,
            enrichment_target,
        )
        .await?;
        for insert in &inserts {
            hold(tx, space_id, &insert.id, run_id, &insert.data, now).await?;
            for source in sources {
                relations::derives_from(tx, space_id, &insert.id, source, lineage, now).await?;
            }
        }
    }
    structure::repository::save_in_transaction(tx, space_id, base, &snapshot).await?;
    let after = structure::history::capture(tx, space_id, &snapshot).await?;
    structure::history::record(tx, space_id, &before, &after).await?;
    Ok(())
}

async fn replace_reserved(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &mut structure::repository::SpaceStructureSnapshot,
    output: &PendingOutput,
    cell: &DerivedCell,
    now: i64,
) -> Result<(), AppError> {
    sqlx::query("UPDATE spaces_nodes SET data=$1,updated_at=$2 WHERE id=$3 AND space_id=$4")
        .bind(sqlx::types::Json(&output.data))
        .bind(now)
        .bind(&output.node_id)
        .bind(space_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    structure::creation::place_existing_content(
        snapshot,
        &output.node_id,
        cell.x,
        cell.y,
        cell.size.clone(),
    )?;
    Ok(())
}

/// Records that a Run owes this node its content, so an output can be settled as failed when its
/// Run ends without publishing it.
async fn hold(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    run_id: &str,
    data: &Value,
    now: i64,
) -> Result<(), AppError> {
    // Which field says the work is still open is a property of the content being made: an asset is
    // produced, a document is derived. Recording it here is what lets failure settle it blindly.
    let state_field = if data.get("derivationState").is_some() {
        "derivationState"
    } else {
        "processingState"
    };
    sqlx::query(
        "INSERT INTO spaces_reserved_outputs(node_id,space_id,run_id,state_field,created_at) \
         VALUES($1,$2,$3,$4,$5)",
    )
    .bind(node_id)
    .bind(space_id)
    .bind(run_id)
    .bind(state_field)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

/// Ends a reservation: the node now holds content of its own, or it has been settled as failed.
pub(crate) async fn release(
    tx: &mut Transaction<'_, Sqlite>,
    node_id: &str,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM spaces_reserved_outputs WHERE node_id=$1")
        .bind(node_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

/// Settles a reservation as failed. Which field says so was decided when the output was reserved,
/// so no caller has to know it again. A node that is no longer reserved is left alone: it either
/// published content of its own or was already settled.
pub(crate) async fn settle_failed(
    tx: &mut Transaction<'_, Sqlite>,
    node_id: &str,
) -> Result<(), String> {
    let reservation: Option<(String, String)> =
        sqlx::query_as("SELECT space_id,state_field FROM spaces_reserved_outputs WHERE node_id=$1")
            .bind(node_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    let Some((space_id, state_field)) = reservation else {
        return Ok(());
    };
    let now = crate::foundation::clock::unix_time_millis()?;
    // Settled even if the node is in the bin: restoring it must not restore work that is over.
    sqlx::query(
        "UPDATE spaces_nodes SET data=json_set(data,'$.'||$1,'failed'),updated_at=$2 \
         WHERE id=$3 AND space_id=$4",
    )
    .bind(&state_field)
    .bind(now)
    .bind(node_id)
    .bind(&space_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    release(tx, node_id).await.map_err(|error| error.message)
}

/// Calls off the Runs that owe content to nodes a person has just deleted, the way deleting a node
/// cancels the enrichment queued for it: work nobody is waiting for stops. The reservations stay,
/// so each is settled when its Run ends — including if the deletion is undone meanwhile.
pub(crate) async fn stop_runs_owing(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_ids: &[String],
    now: i64,
) -> Result<(), String> {
    let runs: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT run_id FROM spaces_reserved_outputs \
         WHERE space_id=$1 AND node_id IN (SELECT value FROM json_each($2)) ORDER BY run_id",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(node_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for run_id in runs {
        crate::spaces::runs::request_cancellation_in_transaction(tx, &run_id, now).await?;
    }
    Ok(())
}

/// Settles outputs whose Run ended without publishing them. A Run can stop at any step, so the
/// reservation is answered by the Run's own outcome rather than by the step that happened to fail:
/// nothing is left waiting on work that is over.
pub(crate) async fn settle_abandoned(pool: &sqlx::SqlitePool) -> Result<(), String> {
    let abandoned: Vec<String> = sqlx::query_scalar(
        "SELECT reserved.node_id FROM spaces_reserved_outputs reserved \
         JOIN spaces_runs run ON run.id=reserved.run_id \
         WHERE run.completed_at IS NOT NULL ORDER BY reserved.created_at,reserved.node_id",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    if abandoned.is_empty() {
        return Ok(());
    }
    let mut tx = db::begin_write(pool).await?;
    for node_id in abandoned {
        settle_failed(&mut tx, &node_id).await?;
    }
    tx.commit().await.map_err(db::sql_error)
}

/// Which of these outputs are already reserved, so they are placed into the set instead of
/// inserted again — and so they do not count as content blocking the set's own placement.
///
/// An identity that is taken by a node someone deleted is neither: reserving would resurrect what
/// a person removed, so the Run is told plainly that its output is gone.
async fn reserved_outputs(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    outputs: &[PendingOutput],
) -> Result<Vec<String>, AppError> {
    let mut reserved = Vec::new();
    for output in outputs {
        match node_state(&mut **tx, space_id, &output.node_id).await? {
            NodeState::Absent => {}
            NodeState::Live => reserved.push(output.node_id.clone()),
            NodeState::Deleted => return Err(OUTPUT_DELETED.into()),
        }
    }
    Ok(reserved)
}

/// What stands at a node identity. Deleted is its own answer: the identity is taken, but the node
/// a Run was going to publish into is gone.
pub(super) enum NodeState {
    Absent,
    Live,
    Deleted,
}

pub(super) async fn node_state<'e, E>(
    executor: E,
    space_id: &str,
    node_id: &str,
) -> Result<NodeState, AppError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let deleted_at: Option<Option<i64>> =
        sqlx::query_scalar("SELECT deleted_at FROM spaces_nodes WHERE id=$1 AND space_id=$2")
            .bind(node_id)
            .bind(space_id)
            .fetch_optional(executor)
            .await
            .map_err(db::sql_error)?;
    Ok(match deleted_at {
        None => NodeState::Absent,
        Some(None) => NodeState::Live,
        Some(Some(_)) => NodeState::Deleted,
    })
}

/// What a reserved output may hold before it publishes: its name, where its asset will live, the
/// size it will be laid out at, and labels a person gave it meanwhile. Everything else it will
/// carry is either established by its own published file or kept of its source by `inheritance`,
/// so a reservation that already carries knowledge has taken it from somewhere it should not.
const RESERVATION_FIELDS: &[&str] = &[
    "kind",
    "title",
    "tags",
    "knowledgeRevision",
    "fileName",
    "assetPath",
    "processingState",
    "intrinsicWidth",
    "intrinsicHeight",
];

pub(crate) fn validate_unpublished_asset(output: &Value) -> Result<&str, String> {
    let output_path = output
        .get("assetPath")
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty())
        .ok_or("Derived output path is unavailable")?;
    let reserved_only = output.as_object().is_some_and(|fields| {
        fields
            .keys()
            .all(|field| RESERVATION_FIELDS.contains(&field.as_str()))
    });
    if output.get("processingState").and_then(Value::as_str) != Some("processing") || !reserved_only
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

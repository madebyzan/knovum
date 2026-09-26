use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Runtime};

use super::edit_contract::{SpaceEditSnapshotResponse, SpacePoint};
use super::{history, StructureLayout, StructureNode};
use crate::spaces::db;

mod payload;
mod persistence;
mod placement;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ImportStructureClipboardRequest {
    space_id: String,
    mutation_id: String,
    expected_edit_revision: i64,
    target_parent_id: Option<String>,
    anchor: SpacePoint,
    payload: ClipboardPayload,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ClipboardPayload {
    version: u8,
    structure: Vec<StructureNode>,
    layouts: Vec<StructureLayout>,
    nodes: Vec<ClipboardNode>,
    relations: Vec<ClipboardRelation>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ClipboardNode {
    id: String,
    data: Value,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ClipboardRelation {
    id: String,
    space_id: String,
    source_id: String,
    target_id: String,
    kind: String,
    rationale: String,
    status: String,
    status_before_invalidation: Option<String>,
    confidence: Option<f64>,
    actor_kind: String,
    actor_id: Option<String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportStructureClipboardResponse {
    snapshot: SpaceEditSnapshotResponse,
    nodes: Vec<Value>,
    relations: Vec<Value>,
    selection_suggestion: Vec<String>,
}

pub(crate) async fn import<R: Runtime>(
    app: &AppHandle<R>,
    request: ImportStructureClipboardRequest,
) -> Result<ImportStructureClipboardResponse, String> {
    if request.space_id.is_empty() || request.space_id.encode_utf16().count() > 128 {
        return Err("Space ID must contain 1 to 128 characters".into());
    }
    if request.expected_edit_revision < 0 {
        return Err("Expected edit revision must be nonnegative".into());
    }
    let digest = super::receipts::digest(&request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    if let Some(result) =
        super::receipts::find_typed(&mut tx, &request.mutation_id, &digest).await?
    {
        tx.commit().await.map_err(db::sql_error)?;
        return Ok(result);
    }
    payload::validate(app, &request.payload)?;
    let (mut snapshot, revision) =
        super::repository::load_in_transaction(&mut tx, &request.space_id).await?;
    if revision != request.expected_edit_revision {
        return Err("space-edit-revision-conflict: Space edit revision conflict".into());
    }
    let before = history::capture(&mut tx, &request.space_id, &snapshot).await?;
    let mut imported = payload::remap(request.payload, &request.anchor)?;
    let selection_suggestion = imported
        .structure
        .iter()
        .filter(|structure_node| structure_node.parent_id().is_none())
        .map(|structure_node| structure_node.id().to_owned())
        .collect();
    placement::attach_roots(
        &mut imported,
        &snapshot,
        request.target_parent_id.as_deref(),
    )?;
    let now = crate::clock::unix_time_millis()?;
    let node_values = persistence::insert_nodes(
        &mut tx,
        &request.space_id,
        &imported.nodes,
        &imported.structure,
        &imported.layouts,
        now,
    )
    .await?;
    let relation_values =
        persistence::insert_relations(&mut tx, &request.space_id, &imported.relations, now).await?;
    snapshot.structure.extend(imported.structure);
    snapshot.layouts.extend(imported.layouts);
    let next =
        super::repository::save_in_transaction(&mut tx, &request.space_id, revision, &snapshot)
            .await?;
    super::portal_integrity::validate_structure_targets(&mut tx).await?;
    let after = history::capture(&mut tx, &request.space_id, &snapshot).await?;
    history::record(
        &mut tx,
        &request.space_id,
        &before,
        &after,
        None,
        None,
        next,
    )
    .await?;
    let result = ImportStructureClipboardResponse {
        snapshot: SpaceEditSnapshotResponse {
            edit_revision: next,
            structure: snapshot.structure,
            layouts: snapshot.layouts,
            relation_styles: snapshot.relation_styles,
        },
        nodes: node_values,
        relations: relation_values,
        selection_suggestion,
    };
    super::receipts::insert_typed(
        &mut tx,
        &request.mutation_id,
        &request.space_id,
        &digest,
        next,
        &result.selection_suggestion,
        &result,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(result)
}

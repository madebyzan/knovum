use super::edit_contract::SpaceEditReceipt;
use super::{history, repository, StructureNode};
use crate::spaces::db;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

mod discard;
mod list;
mod permanent;

pub(crate) use discard::{
    discard_pending_content, DiscardPendingContentRequest, DiscardPendingContentResponse,
};
pub(crate) use list::{list, DeletedRootSummary};
pub(crate) use permanent::{permanently_delete, PermanentlyDeleteStructureRequest};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RestoreDeletedStructureRequest {
    space_id: String,
    mutation_id: String,
    expected_edit_revision: i64,
    batch_id: String,
    root_structure_node_id: Option<String>,
}

pub(crate) async fn restore<R: Runtime>(
    app: &AppHandle<R>,
    request: RestoreDeletedStructureRequest,
) -> Result<SpaceEditReceipt, String> {
    let digest = super::receipts::digest(&request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    if let Some(receipt) =
        super::receipts::find_typed(&mut tx, &request.mutation_id, &digest).await?
    {
        tx.commit().await.map_err(db::sql_error)?;
        return Ok(receipt);
    }
    let (mut snapshot, revision) =
        repository::load_in_transaction(&mut tx, &request.space_id).await?;
    if revision != request.expected_edit_revision {
        return Err("Space edit revision conflict".into());
    }
    let before = history::capture(&mut tx, &request.space_id, &snapshot).await?;
    let restored = match request.root_structure_node_id.as_deref() {
        Some(root) => {
            repository::prepare_restore_root(
                &mut tx,
                &request.space_id,
                &request.batch_id,
                root,
                &uuid::Uuid::new_v4().to_string(),
            )
            .await?
        }
        None => {
            repository::prepare_restore_batch(&mut tx, &request.space_id, &request.batch_id).await?
        }
    };
    let node_ids = restored
        .structure
        .iter()
        .filter_map(|structure_node| match structure_node {
            StructureNode::Content { node_id, .. } => Some(node_id.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let restored_ids = restored
        .structure
        .iter()
        .map(StructureNode::id)
        .collect::<std::collections::HashSet<_>>();
    let selection_suggestion = restored
        .structure
        .iter()
        .filter(|structure_node| {
            structure_node
                .parent_id()
                .map_or(true, |parent| !restored_ids.contains(parent))
        })
        .map(|structure_node| structure_node.id().to_owned())
        .collect::<Vec<_>>();
    let now = crate::clock::unix_time_millis()?;
    crate::spaces::nodes::restore_in_transaction(&mut tx, &request.space_id, &node_ids, now)
        .await?;
    for relation in &restored.relations {
        sqlx::query("UPDATE spaces_relations SET status=$1,status_before_invalidation=$2,updated_at=$3 WHERE id=$4 AND space_id=$5").bind(&relation.status).bind(&relation.status_before_invalidation).bind(now).bind(&relation.id).bind(&request.space_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    }
    repository::merge_restored_snapshot(&mut snapshot, &restored)?;
    let applied =
        repository::save_in_transaction(&mut tx, &request.space_id, revision, &snapshot).await?;
    super::portal_integrity::validate_structure_targets(&mut tx).await?;
    repository::mark_restored(&mut tx, &request.space_id, &restored.batch_ids, now).await?;
    let after = history::capture(&mut tx, &request.space_id, &snapshot).await?;
    history::record(
        &mut tx,
        &request.space_id,
        &before,
        &after,
        Some(&restored.batch_ids[0]),
        Some(true),
        applied,
    )
    .await?;
    let receipt = SpaceEditReceipt {
        mutation_id: request.mutation_id,
        applied_at_revision: applied,
        selection_suggestion,
    };
    super::receipts::insert_typed(
        &mut tx,
        &receipt.mutation_id,
        &request.space_id,
        &digest,
        applied,
        &receipt.selection_suggestion,
        &receipt,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(receipt)
}

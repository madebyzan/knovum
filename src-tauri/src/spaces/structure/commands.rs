use tauri::{AppHandle, Runtime};

use super::edit_contract::{
    ExecuteSpaceEditRequest, SpaceEditHistoryReceipt, SpaceEditHistoryRequest, SpaceEditReceipt,
};
use super::{history, planner, planner_clone, receipts, repository, validation};
use crate::spaces::db;

#[tauri::command]
pub(crate) async fn execute_space_edit<R: Runtime>(
    app: AppHandle<R>,
    request: ExecuteSpaceEditRequest,
) -> Result<SpaceEditReceipt, String> {
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let receipt = execute_in_transaction(&mut tx, request).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(receipt)
}

pub(crate) async fn execute_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: ExecuteSpaceEditRequest,
) -> Result<SpaceEditReceipt, String> {
    validation::validate_id("Space ID", &request.space_id)?;
    validation::validate_id("Mutation ID", &request.mutation_id)?;
    if request.expected_edit_revision < 0 {
        return Err("Expected edit revision must be nonnegative".into());
    }
    let digest = receipts::request_digest(&request)?;
    if let Some(receipt) = receipts::find(tx, &request.mutation_id, &digest).await? {
        return Ok(receipt);
    }
    let (mut snapshot, edit_revision) =
        repository::load_in_transaction(tx, &request.space_id).await?;
    if edit_revision != request.expected_edit_revision {
        return Err("Space edit revision conflict".into());
    }
    super::content_height_read_set::validate(
        &mut *tx,
        &request.space_id,
        &snapshot,
        &request.command,
    )
    .await?;
    let records_history = request.command.records_history();
    let before = if records_history {
        Some(history::capture(&mut *tx, &request.space_id, &snapshot).await?)
    } else {
        None
    };
    let deletion_batch_id = matches!(
        &request.command,
        super::edit_contract::SpaceEditCommand::StructureDeleteSubtrees { .. }
    )
    .then_some(request.mutation_id.as_str());
    let plan = match &request.command {
        super::edit_contract::SpaceEditCommand::StructureClone {
            root_ids,
            target_parent_id,
            offset,
        } => {
            let selection = planner_clone::clone_in_transaction(
                &mut *tx,
                &request.space_id,
                &mut snapshot,
                root_ids,
                target_parent_id.as_deref(),
                offset,
            )
            .await?;
            planner::PlanResult {
                selection,
                deletion: None,
            }
        }
        command => planner::plan(command, &mut snapshot)?,
    };
    if let Some(deletion) = &plan.deletion {
        super::deletion_transaction::apply(tx, &request, &mut snapshot, deletion).await?;
    }
    let applied_at_revision = repository::save_in_transaction(
        &mut *tx,
        &request.space_id,
        request.expected_edit_revision,
        &snapshot,
    )
    .await?;
    super::portal_integrity::validate_structure_targets(&mut *tx).await?;
    if let Some(before) = before {
        let after = history::capture(&mut *tx, &request.space_id, &snapshot).await?;
        history::record(
            &mut *tx,
            &request.space_id,
            &before,
            &after,
            deletion_batch_id,
            deletion_batch_id.map(|_| false),
            applied_at_revision,
        )
        .await?;
    }
    let receipt = SpaceEditReceipt {
        mutation_id: request.mutation_id,
        applied_at_revision,
        selection_suggestion: plan.selection,
    };
    receipts::insert(tx, &request.space_id, &digest, &receipt).await?;
    Ok(receipt)
}

#[tauri::command]
pub(crate) async fn undo_space_edit<R: Runtime>(
    app: AppHandle<R>,
    request: SpaceEditHistoryRequest,
) -> Result<SpaceEditHistoryReceipt, String> {
    apply_history(app, request, false).await
}

#[tauri::command]
pub(crate) async fn redo_space_edit<R: Runtime>(
    app: AppHandle<R>,
    request: SpaceEditHistoryRequest,
) -> Result<SpaceEditHistoryReceipt, String> {
    apply_history(app, request, true).await
}

async fn apply_history<R: Runtime>(
    app: AppHandle<R>,
    request: SpaceEditHistoryRequest,
    redo: bool,
) -> Result<SpaceEditHistoryReceipt, String> {
    validation::validate_id("Space ID", &request.space_id)?;
    if request.expected_edit_revision < 0 {
        return Err("Expected edit revision must be nonnegative".into());
    }
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let (snapshot, revision) = repository::load_in_transaction(&mut tx, &request.space_id).await?;
    if revision != request.expected_edit_revision {
        return Err("Space edit revision conflict".into());
    }
    let applied_at_revision =
        history::apply(&mut tx, &request.space_id, revision, &snapshot, redo).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(SpaceEditHistoryReceipt {
        applied_at_revision,
        selection_suggestion: Vec::new(),
    })
}

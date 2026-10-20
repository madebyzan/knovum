use crate::foundation::app_error::AppError;
use tauri::{AppHandle, Runtime};

use super::edit_contract::{
    ExecuteSpaceEditRequest, SpaceEditHistoryReceipt, SpaceEditHistoryRequest, SpaceEditReceipt,
};
use super::editor_protection::{EditorResourceProtection, SpaceEditorResource};
use super::{history, planner, receipts, repository, validation};
use crate::spaces::db;

#[tauri::command]
pub(crate) async fn execute_space_edit<R: Runtime>(
    app: AppHandle<R>,
    request: ExecuteSpaceEditRequest,
    editor_resources: Vec<SpaceEditorResource>,
) -> Result<SpaceEditReceipt, AppError> {
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let protection = EditorResourceProtection::capture(&mut tx, &editor_resources).await?;
    let receipt = execute_in_transaction(&mut tx, request).await?;
    protection.ensure_preserved(&mut tx).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(receipt)
}

pub(crate) async fn execute_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: ExecuteSpaceEditRequest,
) -> Result<SpaceEditReceipt, AppError> {
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
        return Err(AppError::new(
            "space_edit_revision_conflict",
            "Space edit revision conflict",
        ));
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
    let plan = match &request.command {
        super::edit_contract::SpaceEditCommand::StructureClone {
            root_ids,
            target_parent_id,
            offset,
        } => {
            let selection = planner::clone::clone_in_transaction(
                &mut *tx,
                &request.space_id,
                &mut snapshot,
                root_ids,
                target_parent_id.as_deref(),
                offset,
            )
            .await?;
            planner::StructureEditPlan {
                selection,
                deletion: None,
            }
        }
        command => planner::plan(command, &mut snapshot)?,
    };
    if let super::edit_contract::SpaceEditCommand::ContentUpdateLayout {
        structure_node_id, ..
    } = &request.command
    {
        let node_ids = snapshot
            .structure
            .iter()
            .filter_map(|node| match node {
                super::StructureNode::Content { id, node_id, .. } if id == structure_node_id => {
                    Some(node_id.clone())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        crate::spaces::nodes::fit_content_sizes_in_snapshot(
            &mut *tx,
            &request.space_id,
            &mut snapshot,
            &node_ids,
        )
        .await?;
    }
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
    super::portal_integrity::validate_structure_targets(&mut *tx, &[&request.space_id]).await?;
    if let Some(before) = before {
        let after = history::capture(&mut *tx, &request.space_id, &snapshot).await?;
        history::record(&mut *tx, &request.space_id, &before, &after).await?;
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
    editor_resources: Vec<SpaceEditorResource>,
) -> Result<SpaceEditHistoryReceipt, AppError> {
    apply_history(app, request, editor_resources, false).await
}

#[tauri::command]
pub(crate) async fn redo_space_edit<R: Runtime>(
    app: AppHandle<R>,
    request: SpaceEditHistoryRequest,
    editor_resources: Vec<SpaceEditorResource>,
) -> Result<SpaceEditHistoryReceipt, AppError> {
    apply_history(app, request, editor_resources, true).await
}

async fn apply_history<R: Runtime>(
    app: AppHandle<R>,
    request: SpaceEditHistoryRequest,
    editor_resources: Vec<SpaceEditorResource>,
    redo: bool,
) -> Result<SpaceEditHistoryReceipt, AppError> {
    validation::validate_id("Space ID", &request.space_id)?;
    if request.expected_edit_revision < 0 {
        return Err("Expected edit revision must be nonnegative".into());
    }
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let (snapshot, revision) = repository::load_in_transaction(&mut tx, &request.space_id).await?;
    if revision != request.expected_edit_revision {
        return Err(AppError::new(
            "space_edit_revision_conflict",
            "Space edit revision conflict",
        ));
    }
    let protection = EditorResourceProtection::capture(&mut tx, &editor_resources).await?;
    let applied_at_revision =
        history::apply(&mut tx, &request.space_id, revision, &snapshot, redo).await?;
    protection.ensure_preserved(&mut tx).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(SpaceEditHistoryReceipt {
        applied_at_revision,
        selection_suggestion: Vec::new(),
    })
}

#[tauri::command]
pub(crate) async fn import_structure_clipboard<R: Runtime>(
    app: AppHandle<R>,
    request: super::clipboard::ImportStructureClipboardRequest,
) -> Result<super::clipboard::ImportStructureClipboardResponse, AppError> {
    super::clipboard::import(&app, request).await
}

#[tauri::command]
pub(crate) async fn discard_pending_content<R: Runtime>(
    app: AppHandle<R>,
    request: super::deletion::DiscardPendingContentRequest,
) -> Result<super::deletion::DiscardPendingContentResponse, AppError> {
    super::deletion::discard_pending_content(&app, request).await
}

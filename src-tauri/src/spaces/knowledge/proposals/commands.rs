//! IPC commands of Knowledge Proposals.

use tauri::AppHandle;

use super::{KnowledgeProposal, ProposalAcceptance, ProposalOrigin};
use crate::{foundation::app_error::AppError, spaces::db};

/// Pending proposals of a Space; with `origin`, only those it produced.
#[tauri::command]
pub(crate) async fn list_knowledge_proposals(
    app: AppHandle,
    space_id: String,
    origin: Option<ProposalOrigin>,
) -> Result<Vec<KnowledgeProposal>, AppError> {
    let pool = db::pool(&app).await?;
    super::list(
        &mut *pool.acquire().await.map_err(db::sql_error)?,
        &space_id,
        origin.as_ref(),
    )
    .await
}

/// Accepts a proposal, or the closed part of the chosen changes, as the person using Knovum.
#[tauri::command]
pub(crate) async fn accept_knowledge_proposal(
    app: AppHandle,
    proposal_id: String,
    changes: Option<Vec<usize>>,
) -> Result<ProposalAcceptance, AppError> {
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let space_id = super::space_of(&mut tx, &proposal_id).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let acceptance = super::accept(&mut tx, &proposal_id, changes.as_deref(), now).await?;
    tx.commit().await.map_err(db::sql_error)?;
    super::announce(&app, &space_id);
    Ok(acceptance)
}

/// Dismisses a proposal. The same proposal is not made again.
#[tauri::command]
pub(crate) async fn dismiss_knowledge_proposal(
    app: AppHandle,
    proposal_id: String,
) -> Result<(), AppError> {
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let space_id = super::space_of(&mut tx, &proposal_id).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    super::dismiss(&mut tx, &proposal_id, now).await?;
    tx.commit().await.map_err(db::sql_error)?;
    super::announce(&app, &space_id);
    Ok(())
}

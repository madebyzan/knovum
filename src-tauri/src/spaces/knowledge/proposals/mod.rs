//! Knowledge Proposals: the single path from analysis to the Knowledge graph.
//!
//! A proposal is validated like a commit when it is made, deduplicated by meaning, shown with the
//! evidence it rests on, and accepted, in full or as a closed subset, through the same atomic
//! commit as every other Knowledge change.

pub(crate) mod basis;
mod closure;
pub(crate) mod commands;
mod contracts;
mod fingerprint;
mod repository;

use std::collections::BTreeSet;

use sqlx::{Sqlite, SqliteConnection, Transaction};
use tauri::{AppHandle, Emitter};

pub(crate) use contracts::{
    KnowledgeProposal, ProposalAcceptance, ProposalDraft, ProposalFreshness, ProposalNode,
    ProposalOrigin, ProposalReceipt, ProposalState, ProposedChange,
};
pub(crate) use repository::Proposer;

use super::{action, change_commit::KnowledgeChange, events::EventActor};
use crate::foundation::app_error::AppError;

pub(crate) async fn propose(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    proposer: &Proposer<'_>,
    draft: ProposalDraft,
    now: i64,
) -> Result<ProposalReceipt, AppError> {
    action::validate(tx, space_id, &draft.changes).await?;
    let stored = repository::insert(
        tx,
        space_id,
        proposer,
        &draft,
        &fingerprint::fingerprint(&draft.changes),
        &targets(&draft),
        now,
    )
    .await?;
    Ok(ProposalReceipt {
        proposal_id: stored.id,
        state: stored.state,
    })
}

/// The existing nodes a draft changes, relates or quotes. A temp id names a node the proposal
/// creates, so it is not a target.
fn targets(draft: &ProposalDraft) -> Vec<String> {
    let created: BTreeSet<&str> = draft.changes.iter().filter_map(closure::temp_id).collect();
    let named = draft.changes.iter().flat_map(closure::references);
    let quoted = draft.basis.iter().map(|span| span.node_id.as_str());
    named
        .chain(quoted)
        .filter(|id| !created.contains(id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

pub(crate) async fn list(
    connection: &mut SqliteConnection,
    space_id: &str,
    origin: Option<&ProposalOrigin>,
) -> Result<Vec<KnowledgeProposal>, AppError> {
    let rows = repository::list_pending(connection, space_id, origin).await?;
    let mut proposals = Vec::with_capacity(rows.len());
    for row in rows {
        proposals.push(present(connection, row).await?);
    }
    Ok(proposals)
}

async fn present(
    connection: &mut SqliteConnection,
    row: repository::ProposalRow,
) -> Result<KnowledgeProposal, AppError> {
    let freshness = freshness(connection, &row).await?;
    let nodes = repository::node_titles(connection, &row.id)
        .await?
        .into_iter()
        .map(|(id, title)| ProposalNode { id, title })
        .collect();
    let requires = closure::dependencies(&row.changes);
    Ok(KnowledgeProposal {
        id: row.id,
        space_id: row.space_id,
        origin: row.origin,
        rationale: row.rationale,
        changes: row
            .changes
            .into_iter()
            .zip(requires)
            .map(|(change, requires)| ProposedChange { change, requires })
            .collect(),
        basis: row.basis,
        nodes,
        state: row.state,
        freshness,
        created_at: row.created_at,
    })
}

async fn freshness(
    connection: &mut SqliteConnection,
    row: &repository::ProposalRow,
) -> Result<ProposalFreshness, String> {
    let liveness = repository::liveness(connection, row).await?;
    Ok(if !liveness.targets_live {
        ProposalFreshness::TargetGone
    } else if !liveness.basis_current {
        ProposalFreshness::BasisChanged
    } else {
        ProposalFreshness::Current
    })
}

/// Accepts the proposal, or the closed part of `chosen` (indexes into its changes), as the person
/// using Knovum. A proposal whose evidence or targets changed since is refused: accepting it would
/// record knowledge the sources no longer state.
pub(crate) async fn accept(
    tx: &mut Transaction<'_, Sqlite>,
    proposal_id: &str,
    chosen: Option<&[usize]>,
    now: i64,
) -> Result<ProposalAcceptance, AppError> {
    let row = repository::load(tx, proposal_id).await?;
    if row.state != ProposalState::Pending {
        return Err("knowledge-proposal-decided: the proposal was already decided".into());
    }
    match freshness(tx, &row).await? {
        ProposalFreshness::Current => {}
        ProposalFreshness::BasisChanged => {
            return Err("knowledge-proposal-outdated: a source it quotes has changed".into())
        }
        ProposalFreshness::TargetGone => {
            return Err("knowledge-proposal-outdated: a node it names no longer exists".into())
        }
    }
    let operations = chosen_operations(&row.changes, chosen)?;
    let committed = action::commit(
        tx,
        EventActor::new("human", None),
        &row.space_id,
        operations,
        now,
    )
    .await?;
    let resolved_ids = committed
        .get("resolvedIds")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    repository::decide(
        tx,
        proposal_id,
        ProposalState::Accepted,
        Some(&resolved_ids),
        now,
    )
    .await?;
    Ok(ProposalAcceptance {
        proposal_id: proposal_id.to_owned(),
        resolved_ids,
    })
}

fn chosen_operations(
    changes: &[KnowledgeChange],
    chosen: Option<&[usize]>,
) -> Result<Vec<KnowledgeChange>, String> {
    let all: Vec<usize> = (0..changes.len()).collect();
    let kept = closure::close(changes, chosen.unwrap_or(&all));
    if kept.is_empty() {
        return Err(
            "knowledge-proposal-empty: choose at least one change that can be accepted on its own"
                .into(),
        );
    }
    Ok(kept
        .into_iter()
        .map(|index| changes[index].clone())
        .collect())
}

/// The Space a proposal belongs to.
pub(crate) async fn space_of(
    connection: &mut SqliteConnection,
    proposal_id: &str,
) -> Result<String, AppError> {
    Ok(repository::load(connection, proposal_id).await?.space_id)
}

/// Tells the Space its proposals changed; listeners read them again.
pub(crate) fn announce(app: &AppHandle, space_id: &str) {
    let update = serde_json::json!({ "spaceId": space_id });
    if let Err(error) = app.emit("space-knowledge-proposals-update", update) {
        log::warn!("Could not publish Knowledge Proposal update: {error}");
    }
}

pub(crate) async fn dismiss(
    tx: &mut Transaction<'_, Sqlite>,
    proposal_id: &str,
    now: i64,
) -> Result<(), AppError> {
    Ok(repository::decide(tx, proposal_id, ProposalState::Dismissed, None, now).await?)
}

#[cfg(test)]
mod tests;

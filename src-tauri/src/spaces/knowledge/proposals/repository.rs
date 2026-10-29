//! Storage of Knowledge Proposals. A proposal's meaning is its fingerprint: inserting one the Space
//! already holds, pending or decided, returns the existing one instead of a duplicate.

use serde_json::Value;
use sqlx::{sqlite::SqliteRow, Row, Sqlite, SqliteConnection, Transaction};

use super::contracts::{ProposalDraft, ProposalOrigin, ProposalState};
use crate::spaces::{
    db,
    knowledge::{change_commit::KnowledgeChange, grounding::evidence_span::EvidenceSpan},
};

pub(super) struct ProposalRow {
    pub id: String,
    pub space_id: String,
    pub origin: ProposalOrigin,
    pub rationale: String,
    pub changes: Vec<KnowledgeChange>,
    pub basis: Vec<EvidenceSpan>,
    pub state: ProposalState,
    pub created_at: i64,
}

/// Who proposed it, as Knowledge history records actors.
pub(crate) struct Proposer<'a> {
    pub kind: &'a str,
    pub id: Option<&'a str>,
}

/// The stored proposal for a draft: the new one, or the one with the same meaning.
pub(super) struct Stored {
    pub id: String,
    pub state: ProposalState,
}

pub(super) async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    proposer: &Proposer<'_>,
    draft: &ProposalDraft,
    fingerprint: &str,
    targets: &[String],
    now: i64,
) -> Result<Stored, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let inserted = sqlx::query(
        "INSERT INTO spaces_knowledge_proposals(id,space_id,origin_kind,origin,rationale,changes,basis,
         fingerprint,state,proposer_kind,proposer_id,created_at)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,'pending',$9,$10,$11)
         ON CONFLICT(space_id,fingerprint) DO NOTHING",
    )
    .bind(&id)
    .bind(space_id)
    .bind(draft.origin.kind())
    .bind(sqlx::types::Json(&draft.origin))
    .bind(&draft.rationale)
    .bind(sqlx::types::Json(&draft.changes))
    .bind(sqlx::types::Json(&draft.basis))
    .bind(fingerprint)
    .bind(proposer.kind)
    .bind(proposer.id)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .rows_affected()
        == 1;
    if inserted {
        for node_id in targets {
            sqlx::query("INSERT OR IGNORE INTO spaces_knowledge_proposal_targets(proposal_id,node_id) VALUES($1,$2)")
                .bind(&id)
                .bind(node_id)
                .execute(&mut **tx)
                .await
                .map_err(db::sql_error)?;
        }
    }
    let (id, state): (String, String) = sqlx::query_as(
        "SELECT id,state FROM spaces_knowledge_proposals WHERE space_id=$1 AND fingerprint=$2",
    )
    .bind(space_id)
    .bind(fingerprint)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(Stored {
        id,
        state: ProposalState::parse(&state)?,
    })
}

// One literal per query: sqlx accepts only static SQL.
macro_rules! proposal_query {
    ($predicate:literal) => {
        concat!(
            "SELECT id,space_id,origin,rationale,changes,basis,state,created_at FROM spaces_knowledge_proposals ",
            $predicate
        )
    };
}

pub(super) async fn load(
    connection: &mut SqliteConnection,
    id: &str,
) -> Result<ProposalRow, String> {
    let row = sqlx::query(proposal_query!("WHERE id=$1"))
        .bind(id)
        .fetch_optional(connection)
        .await
        .map_err(db::sql_error)?
        .ok_or("knowledge-proposal-missing: the proposal no longer exists")?;
    decode(&row)
}

/// Pending proposals of a Space, newest first; with `origin`, only those it produced.
pub(super) async fn list_pending(
    connection: &mut SqliteConnection,
    space_id: &str,
    origin: Option<&ProposalOrigin>,
) -> Result<Vec<ProposalRow>, String> {
    let origin = origin
        .map(serde_json::to_value)
        .transpose()
        .map_err(db::json_error)?;
    sqlx::query(proposal_query!(
        "WHERE space_id=$1 AND state='pending' AND ($2 IS NULL OR json(origin)=json($2))
         ORDER BY created_at DESC, rowid DESC"
    ))
    .bind(space_id)
    .bind(origin.map(sqlx::types::Json))
    .fetch_all(connection)
    .await
    .map_err(db::sql_error)?
    .iter()
    .map(decode)
    .collect()
}

/// Settles a pending proposal; a proposal that is no longer pending is not settled twice.
pub(super) async fn decide(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    state: ProposalState,
    resolved_ids: Option<&Value>,
    now: i64,
) -> Result<(), String> {
    let executed = sqlx::query(
        "UPDATE spaces_knowledge_proposals SET state=$1,resolved_ids=$2,decided_at=$3
         WHERE id=$4 AND state='pending'",
    )
    .bind(state.as_str())
    .bind(resolved_ids.map(sqlx::types::Json))
    .bind(now)
    .bind(id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        executed,
        "knowledge-proposal-decided: the proposal was already decided",
    )
}

/// Whether every node the proposal names still exists and every passage it quotes is still
/// current at the revision it quoted.
pub(super) struct Liveness {
    pub targets_live: bool,
    pub basis_current: bool,
}

pub(super) async fn liveness(
    connection: &mut SqliteConnection,
    proposal: &ProposalRow,
) -> Result<Liveness, String> {
    let gone: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_knowledge_proposal_targets t
         LEFT JOIN spaces_nodes n ON n.id=t.node_id AND n.deleted_at IS NULL
         WHERE t.proposal_id=$1 AND n.id IS NULL",
    )
    .bind(&proposal.id)
    .fetch_one(&mut *connection)
    .await
    .map_err(db::sql_error)?;
    let mut basis_current = true;
    for span in &proposal.basis {
        let current: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM spaces_current_knowledge_chunks WHERE id=$1 AND source_revision=$2",
        )
        .bind(&span.chunk_id)
        .bind(&span.source_revision)
        .fetch_optional(&mut *connection)
        .await
        .map_err(db::sql_error)?;
        if current.is_none() {
            basis_current = false;
            break;
        }
    }
    Ok(Liveness {
        targets_live: gone == 0,
        basis_current,
    })
}

/// Titles of the existing nodes a proposal names, for review.
pub(super) async fn node_titles(
    connection: &mut SqliteConnection,
    proposal_id: &str,
) -> Result<Vec<(String, String)>, String> {
    sqlx::query_as(
        "SELECT n.id, COALESCE(NULLIF(trim(json_extract(n.data,'$.title')),''), n.kind)
         FROM spaces_knowledge_proposal_targets t JOIN spaces_nodes n ON n.id=t.node_id
         WHERE t.proposal_id=$1 ORDER BY n.id",
    )
    .bind(proposal_id)
    .fetch_all(connection)
    .await
    .map_err(db::sql_error)
}

fn decode(row: &SqliteRow) -> Result<ProposalRow, String> {
    let json = |column: &str| -> Result<Value, String> {
        let raw: String = row.try_get(column).map_err(db::sql_error)?;
        serde_json::from_str(&raw).map_err(db::json_error)
    };
    let state: String = row.try_get("state").map_err(db::sql_error)?;
    Ok(ProposalRow {
        id: row.try_get("id").map_err(db::sql_error)?,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        origin: serde_json::from_value(json("origin")?).map_err(db::json_error)?,
        rationale: row.try_get("rationale").map_err(db::sql_error)?,
        changes: serde_json::from_value(json("changes")?).map_err(db::json_error)?,
        basis: serde_json::from_value(json("basis")?).map_err(db::json_error)?,
        state: ProposalState::parse(&state)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
    })
}

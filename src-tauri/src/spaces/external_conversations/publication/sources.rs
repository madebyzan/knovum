use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::{
    foundation::app_error::AppError,
    spaces::{db, knowledge::grounding},
};

/// Explicit canvas node handed to the Agent for the turn.
const ATTACHMENT_LINEAGE: &str = "Made by a desktop Agent in a turn given this as input";

/// Space node explicitly cited by the deliverable or, if it carries no markers, by the turn reply.
const CITED_LINEAGE: &str = "Made by a desktop Agent in a turn that cited this source";

#[derive(Clone)]
pub(super) struct TurnSource {
    pub(super) node_id: String,
    pub(super) lineage: &'static str,
}

pub(super) struct ResolvedSources {
    pub(super) placement_sources: Vec<String>,
    pub(super) per_output: Vec<Vec<TurnSource>>,
}

/// Resolves exact `derives-from` sources per deliverable:
/// 1. Explicit canvas node attachments apply to every deliverable of the turn.
/// 2. If a deliverable's own text cites live Space nodes via `[Cn]`, only those cited nodes join
///    its attachments — unrelated nodes mentioned in the chat reply are not mixed in.
/// 3. If a deliverable carries no `[Cn]` markers (e.g. non-text artifacts or plain prose), it
///    falls back to the live Space nodes cited by the completed turn's assistant reply.
pub(super) async fn resolve(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    conversation_id: &str,
    run_id: &str,
    snapshot: &str,
    outputs: &[super::super::staging::StagedOutput],
) -> Result<ResolvedSources, AppError> {
    let attachments = live_sources(
        tx,
        space_id,
        &attachment_node_ids(snapshot)?,
        ATTACHMENT_LINEAGE,
    )
    .await?;
    let reply_candidates = reply_cited_node_ids(tx, conversation_id).await?;
    let reply_cited = live_sources(tx, space_id, &reply_candidates, CITED_LINEAGE).await?;

    let mut per_output = Vec::with_capacity(outputs.len());
    let mut placement_sources = Vec::new();
    for output in outputs {
        let output_cited = match deliverable_text(&output.node.data) {
            Some(text) => {
                let citations = grounding::cite_answer(tx, run_id, text).await?;
                live_sources(
                    tx,
                    space_id,
                    &current_citation_node_ids(&citations),
                    CITED_LINEAGE,
                )
                .await?
            }
            None => Vec::new(),
        };
        let cited = if output_cited.is_empty() {
            &reply_cited
        } else {
            &output_cited
        };
        let mut combined = Vec::new();
        for source in attachments.iter().chain(cited.iter()) {
            if source.node_id == output.node.id {
                continue;
            }
            if !combined
                .iter()
                .any(|existing: &TurnSource| existing.node_id == source.node_id)
            {
                combined.push(source.clone());
            }
            if !placement_sources.iter().any(|id| id == &source.node_id) {
                placement_sources.push(source.node_id.clone());
            }
        }
        per_output.push(combined);
    }

    Ok(ResolvedSources {
        placement_sources,
        per_output,
    })
}

fn attachment_node_ids(snapshot: &str) -> Result<Vec<String>, AppError> {
    let input: Value = serde_json::from_str(snapshot).map_err(db::json_error)?;
    let attachments = input
        .get("attachments")
        .and_then(Value::as_array)
        .ok_or("Agent run has no attachment manifest")?;
    let mut ids = Vec::new();
    for entry in attachments {
        if entry["source"] != "node" {
            continue;
        }
        let node_id = entry["nodeId"]
            .as_str()
            .ok_or("Agent attachment has no canvas node identity")?;
        ids.push(node_id.to_owned());
    }
    Ok(ids)
}

async fn reply_cited_node_ids(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
) -> Result<Vec<String>, AppError> {
    let raw: Option<String> = sqlx::query_scalar(
        "SELECT m.content FROM spaces_conversation_messages m \
         WHERE m.id=(SELECT message_id FROM spaces_external_agent_turns \
           WHERE conversation_id=$1 AND state='completed' ORDER BY created_at DESC LIMIT 1)",
    )
    .bind(conversation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };
    let content: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
    let Some(citations) = content.get("citations").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    Ok(current_citation_node_ids(citations))
}

fn deliverable_text(data: &Value) -> Option<&str> {
    data.get("text")
        .or_else(|| data.get("plainText"))
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn current_citation_node_ids(citations: &[Value]) -> Vec<String> {
    citations
        .iter()
        .filter(|citation| {
            citation
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or("current")
                == "current"
        })
        .filter_map(|citation| citation.get("nodeId").and_then(Value::as_str))
        .map(str::to_owned)
        .collect()
}

async fn live_sources(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    candidates: &[String],
    lineage: &'static str,
) -> Result<Vec<TurnSource>, AppError> {
    let mut resolved = Vec::new();
    for node_id in candidates {
        if resolved
            .iter()
            .any(|source: &TurnSource| source.node_id == *node_id)
        {
            continue;
        }
        let present: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
        )
        .bind(node_id)
        .bind(space_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if present.is_some() {
            resolved.push(TurnSource {
                node_id: node_id.clone(),
                lineage,
            });
        }
    }
    Ok(resolved)
}

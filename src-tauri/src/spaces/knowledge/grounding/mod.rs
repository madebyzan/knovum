mod citations;
pub(crate) mod prompt;
pub(crate) mod references;

pub use citations::{validate_current_citations, validate_historical_citations};

use std::collections::HashSet;

use sqlx::{Row, Sqlite, Transaction};

use super::contracts;
use crate::spaces::db;

pub const MAX_THREAD_TITLE_CHARS: usize = 512;
pub const MAX_SCOPE_NODES: usize = 10_000;
pub const MAX_CITATIONS: usize = 1_000;
pub const MAX_MESSAGE_CONTENT_CHARS: usize = 1_000_000;
pub const MAX_CITATION_TITLE_CHARS: usize = 512;
pub const MAX_CITATION_QUOTE_CHARS: usize = 64_000;
pub const MAX_PROVIDER_MODEL_CHARS: usize = 512;

#[derive(Debug, Clone)]
pub struct GroundedThreadContract {
    pub space_id: String,
    pub scope: String,
    pub node_ids: Vec<String>,
}

pub fn validate_title(title: &str) -> Result<(), String> {
    if title.trim().is_empty() {
        return Err("Grounded AI thread title is required".into());
    }
    contracts::validate_text_limit(title, MAX_THREAD_TITLE_CHARS, "Grounded AI thread title")
}

pub fn validate_scope_shape(scope: &str, node_ids: &[String]) -> Result<(), String> {
    if !matches!(scope, "selection" | "section" | "space" | "all-spaces") {
        return Err("Unsupported grounded AI scope".into());
    }
    if node_ids.len() > MAX_SCOPE_NODES {
        return Err(format!("Grounded AI scope exceeds {MAX_SCOPE_NODES} nodes"));
    }
    if matches!(scope, "space" | "all-spaces") && !node_ids.is_empty() {
        return Err("Space-wide grounded AI scopes must not contain explicit node IDs".into());
    }
    let mut unique = HashSet::with_capacity(node_ids.len());
    if node_ids
        .iter()
        .any(|id| id.trim().is_empty() || !unique.insert(id.as_str()))
    {
        return Err("Grounded AI scope node IDs must be non-empty and unique".into());
    }
    Ok(())
}

pub async fn validate_new_scope_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    scope: &str,
    node_ids: &[String],
) -> Result<(), String> {
    if matches!(scope, "selection" | "section") && node_ids.is_empty() {
        return Err(
            "Selection and Section grounded AI scopes require at least one semantic node".into(),
        );
    }
    validate_scope_nodes(tx, space_id, scope, node_ids).await
}

pub async fn validate_scope_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    scope: &str,
    node_ids: &[String],
) -> Result<(), String> {
    validate_scope_shape(scope, node_ids)?;
    ensure_space_exists(tx, space_id).await?;
    if node_ids.is_empty() {
        return Ok(());
    }
    let node_ids_json = serde_json::to_string(node_ids).map_err(db::json_error)?;
    let matched = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM spaces_semantic_nodes WHERE space_id=$1 AND id IN (SELECT value FROM json_each($2))",
    ).bind(space_id).bind(node_ids_json).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    if matched != node_ids.len() as i64 {
        return Err("Grounded AI scope contains a node that is unavailable in this Space".into());
    }
    Ok(())
}

pub fn validate_role(role: &str) -> Result<(), String> {
    matches!(role, "user" | "assistant")
        .then_some(())
        .ok_or_else(|| "Unsupported grounded AI message role".to_string())
}

pub fn validate_message_metadata(
    content: &str,
    provider: Option<&str>,
    model: Option<&str>,
) -> Result<(), String> {
    contracts::validate_text_limit(
        content,
        MAX_MESSAGE_CONTENT_CHARS,
        "Grounded AI message content",
    )?;
    if let Some(provider) = provider {
        contracts::validate_text_limit(provider, MAX_PROVIDER_MODEL_CHARS, "Grounded AI provider")?;
    }
    if let Some(model) = model {
        contracts::validate_text_limit(model, MAX_PROVIDER_MODEL_CHARS, "Grounded AI model")?;
    }
    Ok(())
}

pub async fn thread_contract(
    tx: &mut Transaction<'_, Sqlite>,
    thread_id: &str,
) -> Result<GroundedThreadContract, String> {
    let row = sqlx::query(
        "SELECT space_id,scope,scope_node_ids FROM spaces_ai_threads WHERE id=$1 LIMIT 1",
    )
    .bind(thread_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| "Grounded AI thread no longer exists".to_string())?;
    let scope: String = row.try_get("scope").map_err(db::sql_error)?;
    let raw_ids: String = row.try_get("scope_node_ids").map_err(db::sql_error)?;
    let node_ids: Vec<String> = serde_json::from_str(&raw_ids).map_err(db::json_error)?;
    validate_scope_shape(&scope, &node_ids)?;
    Ok(GroundedThreadContract {
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        scope,
        node_ids,
    })
}

async fn ensure_space_exists(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(), String> {
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces WHERE id=$1")
        .bind(space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    (exists == 1)
        .then_some(())
        .ok_or_else(|| "Grounded AI Space no longer exists".to_string())
}

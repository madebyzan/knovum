//! The indexed passages a Run read, numbered once per Run.
//!
//! Every search result and node read handed to an Agent carries the passage's
//! `cite` id (`C1`, `C2`, …), whichever Agent asked and however it reached
//! Knovum. An answer cites `[C2]`; when it is stored, the markers resolve to
//! Grounded citations, so the reader opens the exact chunk and locator the
//! Agent saw. Passage text, locator and title are copied from the chunk row,
//! never from the tool payload, and node and chunk ids are stored identities,
//! not references: deleting or editing a source never rewrites what was read.

use serde_json::{json, Value};
use sqlx::{Row, SqliteConnection};

use crate::spaces::db;

/// What a tool returned, and so where its passages are.
#[derive(Clone, Copy)]
pub(crate) enum Reading {
    /// A search projection: an array of matches, each one chunk.
    Search,
    /// A node read: the node with its indexed `chunks`.
    Node,
}

impl Reading {
    pub(crate) fn of_action(action_id: &str) -> Option<Self> {
        match action_id {
            "search-knowledge" => Some(Self::Search),
            "read-node" => Some(Self::Node),
            _ => None,
        }
    }
}

/// Adds the Run's `cite` id to every passage in `result`, numbering passages
/// the Run has not read before.
pub(crate) async fn number_passages(
    connection: &mut SqliteConnection,
    run_id: &str,
    reading: Reading,
    result: &mut Value,
) -> Result<(), String> {
    let passages = match reading {
        Reading::Search => result.as_array_mut(),
        Reading::Node => result.get_mut("chunks").and_then(Value::as_array_mut),
    }
    .ok_or("citation-reading-invalid: the result carries no passages")?;
    let chunk_key = match reading {
        Reading::Search => "chunkId",
        Reading::Node => "id",
    };
    for passage in passages {
        let chunk_id = text(passage, chunk_key)?.to_owned();
        let revision = text(passage, "sourceRevision")?.to_owned();
        if let Some(ordinal) = ordinal(connection, run_id, &chunk_id, &revision).await? {
            passage["cite"] = Value::String(format!("C{ordinal}"));
        }
    }
    Ok(())
}

/// The Grounded citations an answer's `[Cn]` markers refer to, in index
/// order. A marker the Run never issued is left as plain text.
pub(crate) async fn cite_answer(
    connection: &mut SqliteConnection,
    run_id: &str,
    answer: &str,
) -> Result<Vec<Value>, String> {
    let cited = markers(answer);
    if cited.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        "SELECT r.ordinal,r.chunk_id,r.node_id,r.node_title,r.source_revision,r.locator,r.quote,
           EXISTS(SELECT 1 FROM spaces_current_knowledge_chunks c
             WHERE c.id=r.chunk_id AND c.source_revision=r.source_revision) AS current
         FROM spaces_run_citations r
         WHERE r.run_id=$1 AND r.ordinal IN (SELECT value FROM json_each($2))
         ORDER BY r.ordinal",
    )
    .bind(run_id)
    .bind(sqlx::types::Json(&cited))
    .fetch_all(&mut *connection)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(citation).collect()
}

async fn ordinal(
    connection: &mut SqliteConnection,
    run_id: &str,
    chunk_id: &str,
    revision: &str,
) -> Result<Option<i64>, String> {
    sqlx::query_scalar(
        "INSERT INTO spaces_run_citations(run_id,ordinal,chunk_id,source_revision,node_id,node_title,locator,quote,created_at)
         SELECT $1,(SELECT COALESCE(MAX(ordinal),0)+1 FROM spaces_run_citations WHERE run_id=$1),
           c.id,c.source_revision,c.node_id,
           substr(COALESCE(NULLIF(trim(json_extract(n.data,'$.title')),''),n.kind),1,512),
           c.locator,substr(c.text,1,16000),$4
         FROM spaces_current_knowledge_chunks c
         JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id
         WHERE c.id=$2 AND c.source_revision=$3
         ON CONFLICT(run_id,chunk_id,source_revision) DO UPDATE SET ordinal=ordinal
         RETURNING ordinal",
    )
    .bind(run_id)
    .bind(chunk_id)
    .bind(revision)
    .bind(crate::foundation::clock::unix_time_millis()?)
    .fetch_optional(&mut *connection)
    .await
    .map_err(db::sql_error)
}

fn citation(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    let locator: String = row.try_get("locator").map_err(db::sql_error)?;
    let mut citation = json!({
        "index": row.try_get::<i64, _>("ordinal").map_err(db::sql_error)?,
        "nodeTitle": row.try_get::<String, _>("node_title").map_err(db::sql_error)?,
        "sourceRevision": row.try_get::<String, _>("source_revision").map_err(db::sql_error)?,
        "locator": serde_json::from_str::<Value>(&locator).map_err(db::json_error)?,
        "quote": row.try_get::<String, _>("quote").map_err(db::sql_error)?,
        "state": "detached",
    });
    if row.try_get::<bool, _>("current").map_err(db::sql_error)? {
        citation["state"] = "current".into();
        citation["chunkId"] = row
            .try_get::<String, _>("chunk_id")
            .map_err(db::sql_error)?
            .into();
        citation["nodeId"] = row
            .try_get::<String, _>("node_id")
            .map_err(db::sql_error)?
            .into();
    }
    Ok(citation)
}

/// Distinct indexes of `[Cn]` markers, in order of first appearance.
fn markers(answer: &str) -> Vec<i64> {
    let mut cited = Vec::new();
    for (start, _) in answer.match_indices("[C") {
        let digits = answer[start + 2..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        let closed = answer[start + 2 + digits.len()..].starts_with(']');
        if let (true, Ok(index)) = (closed, digits.parse::<i64>()) {
            if index > 0 && !cited.contains(&index) {
                cited.push(index);
            }
        }
    }
    cited
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("citation-reading-invalid: passage {key} is missing"))
}

#[cfg(test)]
mod tests;

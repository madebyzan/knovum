use std::collections::{HashMap, VecDeque};

use rmcp::ErrorData as McpError;
use serde_json::{json, Value};
use sqlx::Row;
use tauri::AppHandle;

use crate::spaces::db;

use super::super::contracts;
use super::{mcp_error, mcp_sql_error};

pub(super) async fn find(
    app: &AppHandle,
    space_id: &str,
    source_id: &str,
    target_id: &str,
    max_depth: usize,
) -> Result<Vec<Value>, McpError> {
    if source_id == target_id {
        return Ok(vec![json!({"nodeId": source_id})]);
    }
    let pool = db::pool(app).await.map_err(mcp_error)?;
    let rows = sqlx::query(
        "SELECT id,source_id,target_id,kind,status,rationale FROM spaces_relations \
         WHERE space_id=$1 AND status IN ('active','contested')",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(mcp_sql_error)?;

    let mut graph: HashMap<String, Vec<(String, Value)>> = HashMap::new();
    for row in rows {
        let source: String = row.get("source_id");
        let target: String = row.get("target_id");
        let kind: String = row.get("kind");
        let edge = json!({
            "relationId": row.get::<String,_>("id"),
            "sourceId": source,
            "targetId": target,
            "kind": kind,
            "status": row.get::<String,_>("status"),
            "rationale": row.get::<String,_>("rationale"),
        });
        graph
            .entry(source.clone())
            .or_default()
            .push((target.clone(), edge.clone()));
        if contracts::is_symmetric_relation_kind(&kind) {
            graph.entry(target).or_default().push((source, edge));
        }
    }

    let mut queue = VecDeque::from([(source_id.to_string(), 0usize)]);
    let mut previous: HashMap<String, (String, Value)> = HashMap::new();
    previous.insert(source_id.to_string(), (String::new(), Value::Null));
    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        for (next, edge) in graph.get(&current).into_iter().flatten() {
            if previous.contains_key(next) {
                continue;
            }
            previous.insert(next.clone(), (current.clone(), edge.clone()));
            if next == target_id {
                return Ok(unwind(previous, source_id, target_id));
            }
            queue.push_back((next.clone(), depth + 1));
        }
    }
    Ok(Vec::new())
}

fn unwind(
    previous: HashMap<String, (String, Value)>,
    source_id: &str,
    target_id: &str,
) -> Vec<Value> {
    let mut current = target_id.to_string();
    let mut reversed = Vec::new();
    while current != source_id {
        let Some((prior, edge)) = previous.get(&current) else {
            return Vec::new();
        };
        reversed.push(json!({
            "traversedFrom": prior,
            "traversedTo": current,
            "relation": edge,
        }));
        current = prior.clone();
    }
    reversed.reverse();
    reversed
}

#[cfg(test)]
mod tests;

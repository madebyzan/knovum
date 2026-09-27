use std::collections::{HashMap, VecDeque};

use serde_json::{json, Value};
use sqlx::Row;
use tauri::AppHandle;

use crate::spaces::{db, knowledge::contracts};

pub(crate) async fn find_relation_path(
    app: &AppHandle,
    space_id: &str,
    source_id: &str,
    target_id: &str,
    max_depth: usize,
) -> Result<Vec<Value>, String> {
    if source_id == target_id {
        return Ok(vec![json!({"nodeId": source_id})]);
    }
    let pool = db::pool(app).await?;
    let rows = sqlx::query(
        "SELECT id,source_id,target_id,kind,status,rationale FROM spaces_relations \
         WHERE space_id=$1 AND status IN ('active','contested')",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let graph = build_graph(rows)?;
    Ok(find_path(&graph, source_id, target_id, max_depth))
}

type Graph = HashMap<String, Vec<(String, Value)>>;

fn build_graph(rows: Vec<sqlx::sqlite::SqliteRow>) -> Result<Graph, String> {
    let mut graph = Graph::new();
    for row in rows {
        let source: String = row.try_get("source_id").map_err(db::sql_error)?;
        let target: String = row.try_get("target_id").map_err(db::sql_error)?;
        let kind: String = row.try_get("kind").map_err(db::sql_error)?;
        let edge = json!({
            "relationId": row.try_get::<String,_>("id").map_err(db::sql_error)?, "sourceId": source, "targetId": target,
            "kind": kind, "status": row.try_get::<String,_>("status").map_err(db::sql_error)?,
            "rationale": row.try_get::<String,_>("rationale").map_err(db::sql_error)?,
        });
        graph
            .entry(source.clone())
            .or_default()
            .push((target.clone(), edge.clone()));
        if contracts::is_symmetric_relation_kind(&kind) {
            graph.entry(target).or_default().push((source, edge));
        }
    }
    Ok(graph)
}

fn find_path(graph: &Graph, source_id: &str, target_id: &str, max_depth: usize) -> Vec<Value> {
    let mut queue = VecDeque::from([(source_id.to_string(), 0usize)]);
    let mut previous = HashMap::from([(source_id.to_string(), (String::new(), Value::Null))]);
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
                return unwind(previous, source_id, target_id);
            }
            queue.push_back((next.clone(), depth + 1));
        }
    }
    Vec::new()
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
        reversed.push(json!({"traversedFrom": prior, "traversedTo": current, "relation": edge}));
        current = prior.clone();
    }
    reversed.reverse();
    reversed
}

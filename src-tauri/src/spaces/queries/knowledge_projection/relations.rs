use std::collections::{HashMap, VecDeque};

use serde_json::{json, Value};
use sqlx::{Row, SqliteConnection};
use tauri::AppHandle;

use crate::spaces::db;

pub(crate) async fn list_relations(
    connection: &mut SqliteConnection,
    space_id: &str,
    node_id: Option<&str>,
    kind: Option<&str>,
    limit: i64,
) -> Result<Value, String> {
    let limit = limit.clamp(1, 200);
    let node_id = node_id.map(str::trim).filter(|value| !value.is_empty());
    let kind = kind.map(str::trim).filter(|value| !value.is_empty());
    let rows = sqlx::query(
        "SELECT r.id, r.source_id, COALESCE(json_extract(s.data,'$.title'), s.kind) AS source_title, \
           r.target_id, COALESCE(json_extract(t.data,'$.title'), t.kind) AS target_title, \
           r.kind, r.status, r.rationale, r.confidence, r.updated_at \
         FROM spaces_relations r \
         JOIN spaces_nodes s ON s.id=r.source_id AND s.space_id=r.space_id AND s.deleted_at IS NULL \
         JOIN spaces_nodes t ON t.id=r.target_id AND t.space_id=r.space_id AND t.deleted_at IS NULL \
         WHERE r.space_id=$1 AND r.status IN ('active','contested') \
           AND ($2 IS NULL OR r.source_id=$2 OR r.target_id=$2) \
           AND ($3 IS NULL OR r.kind=$3) \
         ORDER BY r.updated_at DESC, r.id LIMIT $4",
    )
    .bind(space_id)
    .bind(node_id)
    .bind(kind)
    .bind(limit)
    .fetch_all(&mut *connection)
    .await
    .map_err(db::sql_error)?;

    let relations = rows
        .into_iter()
        .map(|row| {
            let mut item = json!({
                "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
                "sourceId": row.try_get::<String, _>("source_id").map_err(db::sql_error)?,
                "sourceTitle": row.try_get::<String, _>("source_title").map_err(db::sql_error)?,
                "targetId": row.try_get::<String, _>("target_id").map_err(db::sql_error)?,
                "targetTitle": row.try_get::<String, _>("target_title").map_err(db::sql_error)?,
                "kind": row.try_get::<String, _>("kind").map_err(db::sql_error)?,
                "status": row.try_get::<String, _>("status").map_err(db::sql_error)?,
                "rationale": row.try_get::<String, _>("rationale").map_err(db::sql_error)?,
                "updatedAt": row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
            });
            if let Some(confidence) = row
                .try_get::<Option<f64>, _>("confidence")
                .map_err(db::sql_error)?
            {
                item["confidence"] = Value::from(confidence);
            }
            Ok(item)
        })
        .collect::<Result<Vec<Value>, String>>()?;
    Ok(Value::Array(relations))
}

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

struct RelationEdge {
    next_node_id: String,
    payload: Value,
}

type Graph = HashMap<String, Vec<RelationEdge>>;

struct PathPredecessor {
    node_id: String,
    edge: Value,
}

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
        graph.entry(source.clone()).or_default().push(RelationEdge {
            next_node_id: target.clone(),
            payload: edge.clone(),
        });
        if crate::spaces::knowledge::relations::is_symmetric_kind(&kind) {
            graph.entry(target).or_default().push(RelationEdge {
                next_node_id: source,
                payload: edge,
            });
        }
    }
    Ok(graph)
}

fn find_path(graph: &Graph, source_id: &str, target_id: &str, max_depth: usize) -> Vec<Value> {
    let mut queue = VecDeque::from([(source_id.to_string(), 0usize)]);
    let mut predecessor_by_node_id = HashMap::from([(
        source_id.to_string(),
        PathPredecessor {
            node_id: String::new(),
            edge: Value::Null,
        },
    )]);
    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        for RelationEdge {
            next_node_id: next,
            payload: edge,
        } in graph.get(&current).into_iter().flatten()
        {
            if predecessor_by_node_id.contains_key(next) {
                continue;
            }
            predecessor_by_node_id.insert(
                next.clone(),
                PathPredecessor {
                    node_id: current.clone(),
                    edge: edge.clone(),
                },
            );
            if next == target_id {
                return unwind(predecessor_by_node_id, source_id, target_id);
            }
            queue.push_back((next.clone(), depth + 1));
        }
    }
    Vec::new()
}

fn unwind(
    predecessor_by_node_id: HashMap<String, PathPredecessor>,
    source_id: &str,
    target_id: &str,
) -> Vec<Value> {
    let mut current = target_id.to_string();
    let mut reversed = Vec::new();
    while current != source_id {
        let Some(PathPredecessor {
            node_id: prior,
            edge,
        }) = predecessor_by_node_id.get(&current)
        else {
            return Vec::new();
        };
        reversed.push(json!({"traversedFrom": prior, "traversedTo": current, "relation": edge}));
        current = prior.clone();
    }
    reversed.reverse();
    reversed
}

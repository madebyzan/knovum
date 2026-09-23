use std::collections::BTreeSet;

use serde_json::{json, Value};
use sqlx::Row;
use tauri::AppHandle;

use crate::spaces::db;

pub(super) struct ResolvedInputs {
    pub items: Vec<Value>,
    pub nodes: Vec<ResolvedNode>,
}

pub(super) struct ResolvedNode {
    pub id: String,
    pub kind: String,
    pub revision: String,
    pub data: Value,
}

pub(super) async fn resolve(
    app: &AppHandle,
    space_id: &str,
    action_id: &str,
    roles: &[crate::spaces::actions::SpaceActionInputRole],
    snapshot: &Value,
) -> Result<ResolvedInputs, String> {
    let ids = collect_ids(snapshot);
    if ids.is_empty() && !roles.is_empty() {
        return Err("action-input-missing: select at least one source resource".into());
    }
    let pool = db::pool(app).await?;
    let mut nodes = Vec::with_capacity(ids.len());
    for id in ids {
        let row = sqlx::query(
            "SELECT kind,data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
        )
        .bind(&id)
        .bind(space_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?
        .ok_or("action-input-unavailable: source resource is unavailable")?;
        let kind: String = row.try_get("kind").map_err(db::sql_error)?;
        let data: Value =
            serde_json::from_str(&row.try_get::<String, _>("data").map_err(db::sql_error)?)
                .map_err(db::json_error)?;
        let revision = revision(&data)?;
        nodes.push(ResolvedNode {
            id,
            kind,
            revision,
            data,
        });
    }
    order_for_action(action_id, &mut nodes)?;
    let items = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            json!({
                "nodeId": node.id,
                "role": if index == 0 { "primary" } else { "source" },
                "sourceRevision": node.revision,
            })
        })
        .collect();
    Ok(ResolvedInputs { items, nodes })
}

fn collect_ids(value: &Value) -> Vec<String> {
    fn visit(value: &Value, ids: &mut BTreeSet<String>) {
        match value {
            Value::Array(values) => values.iter().for_each(|value| visit(value, ids)),
            Value::Object(values) => {
                for key in ["nodeId", "sourceNodeId"] {
                    if let Some(id) = values.get(key).and_then(Value::as_str) {
                        ids.insert(id.to_owned());
                    }
                }
                for key in ["nodeIds", "inputNodeIds", "inputs", "output", "result"] {
                    if let Some(value) = values.get(key) {
                        visit(value, ids);
                    }
                }
            }
            Value::String(id) if uuid::Uuid::parse_str(id).is_ok() => {
                ids.insert(id.clone());
            }
            _ => {}
        }
    }
    let mut ids = BTreeSet::new();
    visit(value, &mut ids);
    ids.into_iter().take(64).collect()
}

fn revision(data: &Value) -> Result<String, String> {
    ["knowledgeRevision", "sourceRevision"]
        .into_iter()
        .find_map(|key| data.get(key).and_then(Value::as_str).map(str::to_owned))
        .ok_or_else(|| "action-input-unindexed: source has no current revision".into())
}

fn order_for_action(action_id: &str, nodes: &mut [ResolvedNode]) -> Result<(), String> {
    let primary_kind = match action_id {
        "generate-video" | "render-presentation" | "generate-image" => Some("markdown"),
        "extract-subtitles" | "render-subtitled-video" => Some("video"),
        _ => None,
    };
    if let Some(kind) = primary_kind {
        let index = nodes
            .iter()
            .position(|node| node.kind == kind && node.id.ends_with("-output"))
            .or_else(|| nodes.iter().position(|node| node.kind == kind))
            .ok_or_else(|| format!("action-input-invalid: {action_id} requires a {kind} source"))?;
        nodes.swap(0, index);
    }
    if action_id == "render-subtitled-video" && !nodes.iter().any(|node| node.kind == "subtitle") {
        return Err("action-input-invalid: render-subtitled-video requires subtitles".into());
    }
    Ok(())
}

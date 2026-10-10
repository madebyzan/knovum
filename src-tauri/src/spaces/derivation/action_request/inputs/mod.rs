use serde_json::{json, Value};
use sha2::Digest;
use sqlx::Row;
use tauri::AppHandle;

mod reviewed;
use reviewed::reviewed_revisions;

use crate::spaces::{
    actions::input_selection::source_node_ids, db, derivation::inputs::RevisionBasis,
};

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
    let ordered = snapshot.get("orderedInputs").and_then(Value::as_array);
    let reviewed = reviewed_revisions(snapshot)?;
    if let Some(items) = ordered {
        for (index, item) in items.iter().enumerate() {
            let role = item.get("role").and_then(Value::as_str);
            if !matches!(
                role,
                Some("primary" | "source" | "context" | "style-reference")
            ) || (index == 0 && role != Some("primary"))
            {
                return Err("action-input-invalid: ordered input role is invalid".into());
            }
        }
    }
    let ids = if let Some(items) = ordered {
        let ids = items
            .iter()
            .map(|item| {
                item.get("nodeId")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .ok_or("action-input-invalid: ordered input has no resource")
            })
            .collect::<Result<Vec<_>, _>>()?;
        if ids.len() > 64 || ids.len() != ids.iter().collect::<std::collections::HashSet<_>>().len()
        {
            return Err(
                "action-input-invalid: ordered resources must be unique and limited to 64".into(),
            );
        }
        ids
    } else {
        source_node_ids(snapshot)?
    };
    if ids.is_empty() && !roles.is_empty() {
        return Err("action-input-missing: select at least one source resource".into());
    }
    let pool = db::pool(app).await?;
    let mut nodes = Vec::with_capacity(ids.len());
    for (index, id) in ids.into_iter().enumerate() {
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
        let encoded: String = row.try_get("data").map_err(db::sql_error)?;
        let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        let revision = if let Some(item) = ordered.and_then(|items| items.get(index)) {
            let expected = item
                .get("dataDigest")
                .and_then(Value::as_str)
                .ok_or("action-input-invalid: ordered input digest is missing")?;
            if hex::encode(sha2::Sha256::digest(encoded.as_bytes())) != expected {
                return Err(
                    "action-input-stale: resource changed after the Action Run was accepted".into(),
                );
            }
            let basis = RevisionBasis::parse(
                item.get("revisionBasis")
                    .and_then(Value::as_str)
                    .ok_or("action-input-invalid: ordered revision basis is missing")?,
            )?;
            crate::spaces::derivation::input_revision::resolve(&kind, &data, basis)?;
            let field = match basis {
                RevisionBasis::Knowledge => "knowledgeRevision",
                RevisionBasis::Asset | RevisionBasis::Rendered => "sourceRevision",
            };
            data.get(field)
                .and_then(Value::as_str)
                .ok_or("action-input-unindexed: source has no required revision")?
                .to_owned()
        } else {
            revision(&data)?
        };
        if reviewed
            .get(&id)
            .is_some_and(|approved| approved != &revision)
        {
            return Err("action-input-stale: reviewed content changed before the next step".into());
        }
        nodes.push(ResolvedNode {
            id,
            kind,
            revision,
            data,
        });
    }
    if ordered.is_none() {
        order_for_action(action_id, &mut nodes)?;
    }
    let items = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            json!({
                "nodeId": node.id,
                "role": ordered.map_or(if index == 0 { "primary" } else { "source" }, |items| items[index]["role"].as_str().expect("ordered role was validated")),
                "sourceRevision": node.revision,
            })
        })
        .collect();
    Ok(ResolvedInputs { items, nodes })
}

fn revision(data: &Value) -> Result<String, String> {
    ["knowledgeRevision", "sourceRevision"]
        .into_iter()
        .find_map(|key| data.get(key).and_then(Value::as_str).map(str::to_owned))
        .ok_or_else(|| "action-input-unindexed: source has no current revision".into())
}

fn order_for_action(action_id: &str, nodes: &mut [ResolvedNode]) -> Result<(), String> {
    let primary_kind = match action_id {
        "generate-image" => Some("markdown"),
        "add-video-subtitles" => Some("video"),
        _ => None,
    };
    if let Some(kind) = primary_kind {
        let index = nodes
            .iter()
            .position(|node| node.kind == kind)
            .ok_or_else(|| format!("action-input-invalid: {action_id} requires a {kind} source"))?;
        nodes[..=index].rotate_right(1);
    }
    Ok(())
}

mod query;

pub(in crate::spaces) use query::validate_query_filter;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::scalars::required_string;
use crate::spaces::db;

pub(in crate::spaces) async fn validate_same_space_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    target_id: &str,
) -> Result<(), String> {
    if !node_id.is_empty() && node_id == target_id {
        return Err("A projection cannot target itself".into());
    }
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(target_id)
    .bind(space_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    (count == 1)
        .then_some(())
        .ok_or_else(|| "Referenced node must exist in the same Space".to_string())
}

pub(super) async fn validate_portal(
    tx: &mut Transaction<'_, Sqlite>,
    payload: &Value,
) -> Result<(), String> {
    let object = payload
        .as_object()
        .ok_or_else(|| "Portal data must be an object".to_string())?;
    let target_space_id = required_string(object, "targetSpaceId", 128)?;
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces WHERE id=$1")
        .bind(target_space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    if count != 1 {
        return Err("Portal target Space does not exist".into());
    }
    let target_count = ["targetSceneId", "targetNodeId", "targetStructureNodeId"]
        .into_iter()
        .filter(|field| payload.get(*field).is_some())
        .count();
    if target_count > 1 {
        return Err(
            "A Portal can target one Saved View, content node, or structure container".into(),
        );
    }
    if let Some(scene_id) = optional_nonempty_string(
        payload.get("targetSceneId"),
        "Portal target Saved View",
        128,
    )? {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM spaces_scenes WHERE id=$1 AND space_id=$2",
        )
        .bind(scene_id)
        .bind(target_space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if count != 1 {
            return Err("Portal target Saved View does not exist in target Space".into());
        }
    }
    if let Some(node_id) =
        optional_nonempty_string(payload.get("targetNodeId"), "Portal target node", 128)?
    {
        validate_same_space_node(tx, target_space_id, "", node_id).await?;
    }
    if let Some(structure_node_id) = optional_nonempty_string(
        payload.get("targetStructureNodeId"),
        "Portal target structure node",
        128,
    )? {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM spaces_structure_nodes \
             WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL AND kind IN ('group','section')",
        )
        .bind(structure_node_id)
        .bind(target_space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if count != 1 {
            return Err("Portal target structure container does not exist in target Space".into());
        }
    }
    Ok(())
}

fn optional_nonempty_string<'a>(
    value: Option<&'a Value>,
    label: &str,
    max: usize,
) -> Result<Option<&'a str>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let text = value
        .as_str()
        .filter(|text| !text.is_empty())
        .ok_or_else(|| format!("{label} must be a non-empty string"))?;
    if text.encode_utf16().count() > max {
        return Err(format!("{label} exceeds {max} characters"));
    }
    Ok(Some(text))
}

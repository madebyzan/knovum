use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::SpaceSceneInput;
use crate::spaces::{db, knowledge::contracts};

pub(super) async fn validate_scene(
    tx: &mut Transaction<'_, Sqlite>,
    scene: &SpaceSceneInput,
) -> Result<(), String> {
    if scene.title.trim().is_empty()
        || !scene.viewport_x.is_finite()
        || !scene.viewport_y.is_finite()
        || !(0.25..=2.5).contains(&scene.viewport_zoom)
    {
        return Err("Saved View title or viewport is invalid".into());
    }
    let focus_ids = string_array(
        &scene.focus_structure_node_ids,
        "Saved View focus nodes",
        100_000,
    )?;
    for structure_node_id in focus_ids {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM spaces_structure_nodes \
             WHERE space_id=$1 AND id=$2 AND deleted_at IS NULL)",
        )
        .bind(&scene.space_id)
        .bind(structure_node_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if !exists {
            return Err(format!(
                "Saved View references a missing structure node: {structure_node_id}"
            ));
        }
    }
    for kind in string_array(&scene.relation_kinds, "Saved View relation kinds", 32)? {
        contracts::validate_relation_kind(kind)?;
    }
    for status in string_array(&scene.relation_statuses, "Saved View relation statuses", 8)? {
        contracts::validate_relation_status(status)?;
    }
    contracts::validate_query_filter(tx, &scene.space_id, &scene.query_filter).await
}

fn string_array<'a>(value: &'a Value, label: &str, max: usize) -> Result<Vec<&'a str>, String> {
    let values = value
        .as_array()
        .ok_or_else(|| format!("{label} must be an array"))?;
    if values.len() > max {
        return Err(format!("{label} exceeds the maximum of {max}"));
    }
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| format!("{label} must contain non-empty strings"))
        })
        .collect()
}

pub(super) fn json(value: &Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(db::json_error)
}

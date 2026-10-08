use sqlx::{Sqlite, Transaction};

use super::SpaceSceneInput;
use crate::spaces::db;

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
    if scene.focus_structure_node_ids.len() > 100_000
        || scene.focus_structure_node_ids.iter().any(String::is_empty)
        || scene.relation_kinds.len() > 32
        || scene.relation_statuses.len() > 8
    {
        return Err("Saved View filters are invalid".into());
    }
    for structure_node_id in &scene.focus_structure_node_ids {
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
    crate::spaces::knowledge::validation::validate_query_filter(
        tx,
        &scene.space_id,
        &scene.query_filter,
    )
    .await
}
pub(super) fn json(value: &impl serde::Serialize) -> Result<String, String> {
    serde_json::to_string(value).map_err(db::json_error)
}

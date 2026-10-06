use std::collections::HashSet;

use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

pub(crate) async fn validate_structure_targets(
    tx: &mut Transaction<'_, Sqlite>,
    changed_space_ids: &[&str],
) -> Result<(), String> {
    let invalid: Option<String> = sqlx::query_scalar(
        "SELECT p.id FROM spaces_nodes p LEFT JOIN spaces_structure_nodes s \
         ON s.id=json_extract(p.data,'$.targetStructureNodeId') \
         AND s.space_id=json_extract(p.data,'$.targetSpaceId') \
         AND s.deleted_at IS NULL AND s.kind IN ('group','section') \
         WHERE p.id IN (\
           SELECT id FROM spaces_nodes WHERE kind='portal' AND deleted_at IS NULL \
           AND space_id IN (SELECT value FROM json_each($1)) \
           UNION SELECT id FROM spaces_nodes WHERE kind='portal' AND deleted_at IS NULL \
           AND json_extract(data,'$.targetSpaceId') IN (SELECT value FROM json_each($1))\
         ) \
         AND json_type(p.data,'$.targetStructureNodeId')='text' AND s.id IS NULL LIMIT 1",
    )
    .bind(serde_json::to_string(changed_space_ids).map_err(db::json_error)?)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    match invalid {
        Some(id) => Err(format!("Portal {id} targets a missing structure container")),
        None => Ok(()),
    }
}

pub(crate) async fn reject_structure_targets(
    tx: &mut Transaction<'_, Sqlite>,
    target_space_id: &str,
    target_structure_node_ids: &HashSet<&str>,
) -> Result<(), String> {
    let portal: Option<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_nodes WHERE kind='portal' AND deleted_at IS NULL \
         AND json_extract(data,'$.targetSpaceId')=$1 \
         AND json_extract(data,'$.targetStructureNodeId') IN (SELECT value FROM json_each($2)) LIMIT 1",
    )
    .bind(target_space_id)
    .bind(serde_json::to_string(target_structure_node_ids).map_err(db::json_error)?)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    match portal {
        Some(id) => Err(format!(
            "Portal {id} must be removed before permanently deleting its target container"
        )),
        None => Ok(()),
    }
}

pub(crate) async fn reject_node_targets(
    tx: &mut Transaction<'_, Sqlite>,
    target_space_id: &str,
    target_node_ids: &HashSet<&str>,
) -> Result<(), String> {
    let portal: Option<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_nodes WHERE kind='portal' AND deleted_at IS NULL \
         AND json_extract(data,'$.targetSpaceId')=$1 \
         AND json_extract(data,'$.targetNodeId') IN (SELECT value FROM json_each($2)) LIMIT 1",
    )
    .bind(target_space_id)
    .bind(serde_json::to_string(target_node_ids).map_err(db::json_error)?)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    match portal {
        Some(id) => Err(format!(
            "Portal {id} must be removed before discarding its target content"
        )),
        None => Ok(()),
    }
}

use sqlx::{Sqlite, Transaction};

use crate::spaces::{db, knowledge::semantic_nodes};

pub(super) fn validate_relation_lifecycle(
    status: &str,
    previous: Option<&str>,
) -> Result<(), String> {
    match (status, previous) {
        ("invalidated", Some("active" | "contested")) | ("active" | "contested", None) => Ok(()),
        _ => Err("Space package Relation lifecycle state is invalid".into()),
    }
}

pub(super) async fn validate_provenance_target(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    kind: &str,
    target_id: &str,
) -> Result<(), String> {
    match kind {
        "node" => semantic_nodes::require(tx, space_id, target_id, "Provenance target").await,
        "relation" => {
            let count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM spaces_relations WHERE id=$1 AND space_id=$2",
            )
            .bind(target_id)
            .bind(space_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(db::sql_error)?;
            (count == 1).then_some(()).ok_or_else(|| {
                "Provenance target Relation does not exist in this Space".to_string()
            })
        }
        _ => Err("Unsupported provenance target kind".into()),
    }
}

pub(super) async fn validate_event_entity(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    kind: &str,
    entity_id: &str,
) -> Result<(), String> {
    let sql = match kind {
        "node" => {
            "SELECT COUNT(*) FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL"
        }
        "relation" => "SELECT COUNT(*) FROM spaces_relations WHERE id=$1 AND space_id=$2",
        "provenance" => "SELECT COUNT(*) FROM spaces_provenance WHERE id=$1 AND space_id=$2",
        "type" => {
            "SELECT COUNT(*) FROM spaces_object_types WHERE id=$1 AND space_id=$2 AND system=0"
        }
        _ => return Err("Unsupported knowledge event entity kind".into()),
    };
    let count = sqlx::query_scalar::<_, i64>(sql)
        .bind(entity_id)
        .bind(space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    (count == 1).then_some(()).ok_or_else(|| {
        "Knowledge event references an entity that is not present in the package".into()
    })
}

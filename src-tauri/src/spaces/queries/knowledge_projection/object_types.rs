use serde_json::{json, Value};
use sqlx::{Row, SqliteConnection};

use crate::spaces::{
    db,
    knowledge::{
        contracts::{RelationKind, RelationStatus},
        validation::valid_statuses_for_object_type,
    },
};

const VALID_RELATION_KINDS: &[RelationKind] = &[
    RelationKind::Reference,
    RelationKind::Related,
    RelationKind::Supports,
    RelationKind::Contradicts,
    RelationKind::DependsOn,
    RelationKind::Supersedes,
    RelationKind::Answers,
    RelationKind::Uses,
    RelationKind::DerivesFrom,
    RelationKind::CausedBy,
];

const VALID_RELATION_STATUSES: &[RelationStatus] =
    &[RelationStatus::Active, RelationStatus::Contested];

/// Lists system and custom knowledge object types available in `space_id`, along with the
/// authoritative status and relation vocabularies accepted by `commit_knowledge_changes`.
pub(crate) async fn list_object_types(
    connection: &mut SqliteConnection,
    space_id: &str,
) -> Result<Value, String> {
    let rows = sqlx::query(
        "SELECT id, name, description, fields, system \
         FROM spaces_object_types \
         WHERE system=1 OR space_id=$1 \
         ORDER BY system DESC, name COLLATE NOCASE",
    )
    .bind(space_id)
    .fetch_all(&mut *connection)
    .await
    .map_err(db::sql_error)?;

    let object_types = rows
        .into_iter()
        .map(|row| {
            let id: String = row.try_get("id").map_err(db::sql_error)?;
            let fields: String = row.try_get("fields").map_err(db::sql_error)?;
            let valid_statuses = valid_statuses_for_object_type(&id);
            Ok(json!({
                "id": id,
                "name": row.try_get::<String, _>("name").map_err(db::sql_error)?,
                "description": row.try_get::<String, _>("description").map_err(db::sql_error)?,
                "fields": serde_json::from_str::<Value>(&fields).map_err(db::json_error)?,
                "validStatuses": valid_statuses,
                "system": row.try_get::<i64, _>("system").map_err(db::sql_error)? != 0,
            }))
        })
        .collect::<Result<Vec<Value>, String>>()?;

    Ok(json!({
        "spaceId": space_id,
        "objectTypes": object_types,
        "validRelationKinds": VALID_RELATION_KINDS.iter().map(|kind| kind.as_str()).collect::<Vec<_>>(),
        "validRelationStatuses": VALID_RELATION_STATUSES.iter().map(|status| status.as_str()).collect::<Vec<_>>(),
    }))
}

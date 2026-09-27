use crate::spaces::db;
use serde::Serialize;
use sqlx::{Row, Sqlite, Transaction};
use ts_rs::TS;

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpaceRelationProjection {
    pub id: String,
    pub space_id: String,
    pub source_id: String,
    pub target_id: String,
    pub kind: String,
    pub rationale: String,
    pub status: String,
    pub status_before_invalidation: Option<String>,
    pub confidence: Option<f64>,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub(crate) async fn list_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<SpaceRelationProjection>, String> {
    let rows = sqlx::query(
        "SELECT id, space_id, source_id, target_id, kind, rationale, status, status_before_invalidation, confidence, \
         actor_kind, actor_id, created_at, updated_at FROM spaces_relations \
         WHERE space_id = ? AND status <> 'invalidated' ORDER BY created_at",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(decode_relation_projection).collect()
}

fn decode_relation_projection(
    row: sqlx::sqlite::SqliteRow,
) -> Result<SpaceRelationProjection, String> {
    Ok(SpaceRelationProjection {
        id: row.try_get::<String, _>("id").map_err(db::sql_error)?,
        space_id: row
            .try_get::<String, _>("space_id")
            .map_err(db::sql_error)?,
        source_id: row
            .try_get::<String, _>("source_id")
            .map_err(db::sql_error)?,
        target_id: row
            .try_get::<String, _>("target_id")
            .map_err(db::sql_error)?,
        kind: row.try_get::<String, _>("kind").map_err(db::sql_error)?,
        rationale: row
            .try_get::<String, _>("rationale")
            .map_err(db::sql_error)?,
        status: row.try_get::<String, _>("status").map_err(db::sql_error)?,
        status_before_invalidation: row
            .try_get::<Option<String>, _>("status_before_invalidation")
            .map_err(db::sql_error)?,
        confidence: row
            .try_get::<Option<f64>, _>("confidence")
            .map_err(db::sql_error)?,
        actor_kind: row
            .try_get::<String, _>("actor_kind")
            .map_err(db::sql_error)?,
        actor_id: row
            .try_get::<Option<String>, _>("actor_id")
            .map_err(db::sql_error)?,
        created_at: row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
        updated_at: row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
    })
}

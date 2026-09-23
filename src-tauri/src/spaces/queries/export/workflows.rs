use serde_json::{json, Value};
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn drafts(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT id,revision,definition,created_at,updated_at FROM spaces_workflow_drafts
         WHERE space_id=$1 ORDER BY created_at,id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.iter()
        .map(|row| {
            Ok(json!({
                "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
                "revision": row.try_get::<i64, _>("revision").map_err(db::sql_error)?,
                "definition": parse(row, "definition")?,
                "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
                "updatedAt": row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
            }))
        })
        .collect()
}

pub(super) async fn versions(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT workflow_id,version,definition,published_at FROM spaces_workflow_versions WHERE space_id=$1
         ORDER BY workflow_id,version",
    ).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    rows.iter()
        .map(|row| {
            Ok(json!({
                "workflowId": row.try_get::<String, _>("workflow_id").map_err(db::sql_error)?,
                "version": row.try_get::<i64, _>("version").map_err(db::sql_error)?,
                "definition": parse(row, "definition")?,
                "publishedAt": row.try_get::<i64, _>("published_at").map_err(db::sql_error)?,
            }))
        })
        .collect()
}

fn parse(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<Value, String> {
    serde_json::from_str(&row.try_get::<String, _>(key).map_err(db::sql_error)?)
        .map_err(db::json_error)
}

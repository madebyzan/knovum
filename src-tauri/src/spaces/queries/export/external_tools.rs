use serde_json::{json, Value};
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn rows(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query("SELECT id,name,endpoint,server_identity,tool_name,input_schema,output_schema,schema_fingerprint,effect,reviewed_at,created_at,updated_at FROM spaces_external_tool_bindings WHERE space_id=$1 ORDER BY created_at,id")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    rows.iter().map(|row| Ok(json!({
        "id": text(row,"id")?, "name": text(row,"name")?, "endpoint": text(row,"endpoint")?,
        "serverIdentity": parsed(row,"server_identity")?, "toolName": text(row,"tool_name")?,
        "inputSchema": parsed(row,"input_schema")?, "outputSchema": optional_parsed(row,"output_schema")?,
        "schemaFingerprint": text(row,"schema_fingerprint")?, "effect": text(row,"effect")?,
        "reviewedAt": integer(row,"reviewed_at")?, "createdAt": integer(row,"created_at")?,
        "updatedAt": integer(row,"updated_at")?,
    }))).collect()
}

fn text(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<String, String> {
    row.try_get(key).map_err(db::sql_error)
}
fn integer(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<i64, String> {
    row.try_get(key).map_err(db::sql_error)
}
fn parsed(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<Value, String> {
    serde_json::from_str(&text(row, key)?).map_err(db::json_error)
}
fn optional_parsed(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<Option<Value>, String> {
    row.try_get::<Option<String>, _>(key)
        .map_err(db::sql_error)?
        .map(|value| serde_json::from_str(&value).map_err(db::json_error))
        .transpose()
}

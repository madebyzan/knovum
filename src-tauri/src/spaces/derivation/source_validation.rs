use crate::spaces::db;
use serde_json::Value;
use sqlx::Row;

use super::input_revision::{self, RevisionBasis};

pub async fn validate(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_id: &str,
    expected: &str,
    basis: RevisionBasis,
) -> Result<(), String> {
    let row = sqlx::query(
        "SELECT kind,data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Selected derivation source is unavailable")?;
    let kind: String = row.try_get("kind").map_err(db::sql_error)?;
    let encoded: String = row.try_get("data").map_err(db::sql_error)?;
    let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    let (revision, _) = input_revision::resolve(&kind, &data, basis)?;
    if revision == expected {
        Ok(())
    } else {
        Err("Selected derivation source changed".into())
    }
}

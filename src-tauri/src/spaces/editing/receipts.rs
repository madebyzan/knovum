use serde::Serialize;
use sqlx::Row;
use crate::spaces::db;
use super::contracts::{Receipt, ReceiptChanges};
pub(super) fn validate_request(space_id: &str, mutation_id: &str, revision: i64) -> Result<(), String> {
    if !crate::spaces::canvas_state::valid_structure_id(space_id) || uuid::Uuid::parse_str(mutation_id).is_err() || !(0..9_007_199_254_740_991).contains(&revision) { return Err("Invalid editor identity, mutation identity or revision".into()); }
    Ok(())
}
pub(super) fn fingerprint(value: &impl Serialize) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(db::json_error)?;
    if bytes.len() > 128 * 1024 * 1024 { return Err("Editor request exceeds its transfer limit".into()); }
    let digest = ring::digest::digest(&ring::digest::SHA256, &bytes);
    Ok(digest.as_ref().iter().map(|byte| format!("{byte:02x}")).collect())
}
pub(super) async fn find(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, mutation_id: &str) -> Result<Option<Receipt>, String> {
    let row = sqlx::query("SELECT mutation_id,request_hash,applied_revision,changes,selection_before,selection_after,invalidated_at FROM spaces_edit_receipts WHERE space_id=$1 AND mutation_id=$2")
        .bind(space_id).bind(mutation_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
    let Some(row) = row else { return Ok(None); };
    if row.try_get::<Option<i64>,_>("invalidated_at").map_err(db::sql_error)?.is_some() { return Err("This edit refers to permanently removed data and cannot be replayed".into()); }
    let changes: ReceiptChanges = serde_json::from_str(&row.try_get::<String,_>("changes").map_err(db::sql_error)?).map_err(db::json_error)?;
    if changes.version != 1 { return Err("Unsupported edit receipt".into()); }
    Ok(Some(Receipt { mutation_id: row.try_get("mutation_id").map_err(db::sql_error)?, request_hash: row.try_get("request_hash").map_err(db::sql_error)?, revision: row.try_get("applied_revision").map_err(db::sql_error)?, changes,
        selection_before: serde_json::from_str(&row.try_get::<String,_>("selection_before").map_err(db::sql_error)?).map_err(db::json_error)?,
        selection_after: serde_json::from_str(&row.try_get::<String,_>("selection_after").map_err(db::sql_error)?).map_err(db::json_error)?,
    }))
}
pub(super) async fn store(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, receipt: &Receipt, now: i64) -> Result<(), String> {
    sqlx::query("INSERT INTO spaces_edit_receipts(space_id,mutation_id,request_hash,applied_revision,changes,selection_before,selection_after,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(space_id).bind(&receipt.mutation_id).bind(&receipt.request_hash).bind(receipt.revision)
        .bind(serde_json::to_string(&receipt.changes).map_err(db::json_error)?).bind(serde_json::to_string(&receipt.selection_before).map_err(db::json_error)?).bind(serde_json::to_string(&receipt.selection_after).map_err(db::json_error)?).bind(now)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}

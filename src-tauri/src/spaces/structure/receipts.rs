use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Row, Sqlite, Transaction};

use super::edit_contract::{ExecuteSpaceEditRequest, SpaceEditReceipt};
use crate::spaces::db;

pub(super) fn request_digest(request: &ExecuteSpaceEditRequest) -> Result<String, String> {
    digest(request)
}

pub(crate) fn digest<T: Serialize>(request: &T) -> Result<String, String> {
    let encoded = serde_json::to_vec(request).map_err(db::json_error)?;
    Ok(hex::encode(Sha256::digest(encoded)))
}

pub(crate) async fn find_typed<T: DeserializeOwned>(
    tx: &mut Transaction<'_, Sqlite>,
    mutation_id: &str,
    digest: &str,
) -> Result<Option<T>, String> {
    let row = sqlx::query(
        "SELECT request_digest,committed_projection FROM spaces_edit_receipts WHERE mutation_id=$1",
    )
    .bind(mutation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some(row) = row else { return Ok(None) };
    let stored: String = row.try_get("request_digest").map_err(db::sql_error)?;
    if stored != digest {
        return Err(
            "mutation-id-conflict: Mutation ID was already used for a different request".into(),
        );
    }
    let value: String = row.try_get("committed_projection").map_err(db::sql_error)?;
    Ok(Some(serde_json::from_str(&value).map_err(db::json_error)?))
}

pub(crate) async fn load_committed_projection(
    pool: &sqlx::SqlitePool,
    mutation_id: &str,
) -> Result<Option<serde_json::Value>, String> {
    let raw: Option<String> = sqlx::query_scalar(
        "SELECT committed_projection FROM spaces_edit_receipts WHERE mutation_id=$1",
    )
    .bind(mutation_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    raw.map(|value| serde_json::from_str(&value).map_err(db::json_error))
        .transpose()
}

pub(crate) async fn insert_typed<T: Serialize>(
    tx: &mut Transaction<'_, Sqlite>,
    mutation_id: &str,
    space_id: &str,
    digest: &str,
    applied_revision: i64,
    selection: &[String],
    committed_projection: &T,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_edit_receipts(mutation_id,space_id,request_digest,applied_revision,selection_suggestion,committed_projection,created_at) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(mutation_id).bind(space_id).bind(digest).bind(applied_revision).bind(sqlx::types::Json(selection)).bind(sqlx::types::Json(committed_projection)).bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}

pub(super) async fn find(
    tx: &mut Transaction<'_, Sqlite>,
    mutation_id: &str,
    digest: &str,
) -> Result<Option<SpaceEditReceipt>, String> {
    let row = sqlx::query(
        "SELECT request_digest,applied_revision,selection_suggestion \
         FROM spaces_edit_receipts WHERE mutation_id=$1",
    )
    .bind(mutation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some(row) = row else { return Ok(None) };
    let stored_digest: String = row.try_get("request_digest").map_err(db::sql_error)?;
    if stored_digest != digest {
        return Err(
            "mutation-id-conflict: Mutation ID was already used for a different request".into(),
        );
    }
    let selection: String = row.try_get("selection_suggestion").map_err(db::sql_error)?;
    Ok(Some(SpaceEditReceipt {
        mutation_id: mutation_id.to_owned(),
        applied_at_revision: row.try_get("applied_revision").map_err(db::sql_error)?,
        selection_suggestion: serde_json::from_str(&selection).map_err(db::json_error)?,
    }))
}

pub(super) async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    digest: &str,
    receipt: &SpaceEditReceipt,
) -> Result<(), String> {
    insert_typed(
        tx,
        &receipt.mutation_id,
        space_id,
        digest,
        receipt.applied_at_revision,
        &receipt.selection_suggestion,
        receipt,
    )
    .await
}

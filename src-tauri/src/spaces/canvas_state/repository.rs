use sqlx::Row;

use super::{contract::CanvasDocumentV1, validation::validate_document};
use crate::spaces::db;

pub(super) async fn load(
    pool: &sqlx::SqlitePool,
    space_id: &str,
) -> Result<Option<(CanvasDocumentV1, i64)>, String> {
    let row = sqlx::query("SELECT document, revision FROM spaces_canvas_state WHERE space_id = $1")
        .bind(space_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?;
    row.map(decode).transpose()
}

pub(super) async fn save(
    pool: &sqlx::SqlitePool,
    space_id: &str,
    base_revision: i64,
    document: &CanvasDocumentV1,
) -> Result<i64, String> {
    let mut tx = db::begin_write(pool).await?;
    let next_revision = save_in_transaction(&mut tx, space_id, base_revision, document).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(next_revision)
}

pub(in crate::spaces) async fn save_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    base_revision: i64,
    document: &CanvasDocumentV1,
) -> Result<i64, String> {
    validate_document(document)?;
    let next_revision = base_revision + 1;
    let encoded = serde_json::to_string(document).map_err(db::json_error)?;
    let now = crate::clock::unix_time_millis()?;
    let changed = if base_revision == 0 {
        sqlx::query(
            "INSERT INTO spaces_canvas_state(space_id,document,revision,updated_at) \
             SELECT $1,$2,1,$3 WHERE EXISTS(SELECT 1 FROM spaces WHERE id=$1) \
             ON CONFLICT(space_id) DO NOTHING",
        )
        .bind(space_id)
        .bind(encoded)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?
    } else {
        sqlx::query(
            "UPDATE spaces_canvas_state SET document=$1,revision=$2,updated_at=$3 \
             WHERE space_id=$4 AND revision=$5",
        )
        .bind(encoded)
        .bind(next_revision)
        .bind(now)
        .bind(space_id)
        .bind(base_revision)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?
    };
    if changed.rows_affected() != 1 {
        return Err("Canvas state revision conflict".into());
    }
    crate::spaces::assets::synchronize_canvas_references(tx, space_id, document).await?;
    Ok(next_revision)
}

fn decode(row: sqlx::sqlite::SqliteRow) -> Result<(CanvasDocumentV1, i64), String> {
    let document = row
        .try_get::<String, _>("document")
        .map_err(db::sql_error)?;
    let document = serde_json::from_str(&document).map_err(db::json_error)?;
    validate_document(&document)?;
    Ok((document, row.try_get("revision").map_err(db::sql_error)?))
}

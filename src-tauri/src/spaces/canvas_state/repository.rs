use sqlx::Row;

use super::{contract::SpaceEditorDocument, validation::validate_document};
use crate::spaces::db;

pub(super) async fn load(
    pool: &sqlx::SqlitePool,
    space_id: &str,
) -> Result<Option<(SpaceEditorDocument, i64)>, String> {
    let row = sqlx::query("SELECT document, revision FROM spaces_canvas_state WHERE space_id = $1")
        .bind(space_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?;
    if row.is_none() {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL").bind(space_id).fetch_one(pool).await.map_err(db::sql_error)?;
        if count != 0 { return Err("Active content is missing its editor organization document".into()); }
    }
    row.map(decode).transpose()
}

pub(super) async fn save(
    pool: &sqlx::SqlitePool,
    space_id: &str,
    base_revision: i64,
    document: &SpaceEditorDocument,
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
    document: &SpaceEditorDocument,
) -> Result<i64, String> {
    validate_document(document)?;
    if !(0..9_007_199_254_740_991).contains(&base_revision) { return Err("Invalid editor revision".into()); }
    super::validation::validate_content_references(tx, space_id, document).await?;
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

fn decode(row: sqlx::sqlite::SqliteRow) -> Result<(SpaceEditorDocument, i64), String> {
    let document = row
        .try_get::<String, _>("document")
        .map_err(db::sql_error)?;
    let document = serde_json::from_str(&document).map_err(db::json_error)?;
    validate_document(&document)?;
    Ok((document, row.try_get("revision").map_err(db::sql_error)?))
}

pub(in crate::spaces) async fn load_in_transaction(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str) -> Result<(SpaceEditorDocument, i64), String> {
    let row = sqlx::query("SELECT document,revision FROM spaces_canvas_state WHERE space_id=$1").bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
    match row { Some(row) => decode(row), None => Ok((SpaceEditorDocument::empty(), 0)) }
}

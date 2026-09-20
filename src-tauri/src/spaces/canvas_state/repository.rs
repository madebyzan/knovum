use sqlx::Row;
use crate::spaces::db;
use super::{contract::{SpaceEditorDocument, CanvasViewport}, stored_document::StoredEditorDocument};
pub(super) async fn load(pool: &sqlx::SqlitePool, space_id: &str) -> Result<Option<(SpaceEditorDocument, i64)>, String> {
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces WHERE id=$1)").bind(space_id).fetch_one(&mut *tx).await.map_err(db::sql_error)?;
    if !exists { tx.commit().await.map_err(db::sql_error)?; return Ok(None); }
    let state = load_in_transaction(&mut tx, space_id).await?;
    super::validation::validate_content_references(&mut tx, space_id, &state.0).await?;
    tx.commit().await.map_err(db::sql_error)?; Ok(Some(state))
}
pub(in crate::spaces) async fn load_in_transaction(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str) -> Result<(SpaceEditorDocument, i64), String> {
    let row = sqlx::query("SELECT document,revision FROM spaces_canvas_state WHERE space_id=$1").bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
    let viewport = super::viewport::read(tx, space_id).await?;
    match row { Some(row) => decode(row, viewport), None => { let mut empty = SpaceEditorDocument::empty(); empty.viewport = viewport; Ok((empty,0)) } }
}
pub(super) async fn save(pool: &sqlx::SqlitePool, space_id: &str, base_revision: i64, document: &SpaceEditorDocument) -> Result<i64, String> {
    let mut tx = db::begin_write(pool).await?;
    let revision = save_in_transaction(&mut tx, space_id, base_revision, document).await?;
    tx.commit().await.map_err(db::sql_error)?; Ok(revision)
}
pub(in crate::spaces) async fn save_in_transaction(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, base_revision: i64, document: &SpaceEditorDocument) -> Result<i64, String> {
    if !(0..9_007_199_254_740_991).contains(&base_revision) { return Err("Invalid edit revision".into()); }
    super::validate_document(document)?;
    super::validation::validate_content_references(tx, space_id, document).await?;
    let revision = base_revision + 1;
    let encoded = serde_json::to_string(&StoredEditorDocument::from_document(document)).map_err(db::json_error)?;
    let now = crate::clock::unix_time_millis()?;
    let result = if base_revision == 0 {
        sqlx::query("INSERT INTO spaces_canvas_state(space_id,document,revision,updated_at) SELECT $1,$2,$3,$4 WHERE NOT EXISTS(SELECT 1 FROM spaces_canvas_state WHERE space_id=$1)")
            .bind(space_id).bind(encoded).bind(revision).bind(now).execute(&mut **tx).await
    } else {
        sqlx::query("UPDATE spaces_canvas_state SET document=$1,revision=$2,updated_at=$3 WHERE space_id=$4 AND revision=$5")
            .bind(encoded).bind(revision).bind(now).bind(space_id).bind(base_revision).execute(&mut **tx).await
    }.map_err(db::sql_error)?;
    if result.rows_affected() != 1 { return Err("Space edit revision conflict".into()); }
    crate::spaces::assets::synchronize_canvas_references(tx, space_id, document).await?;
    Ok(revision)
}
fn decode(row: sqlx::sqlite::SqliteRow, viewport: CanvasViewport) -> Result<(SpaceEditorDocument, i64), String> {
    let encoded: String = row.try_get("document").map_err(db::sql_error)?;
    let document = serde_json::from_str::<StoredEditorDocument>(&encoded).map_err(db::json_error)?.into_document(viewport);
    super::validate_document(&document)?;
    let revision: i64 = row.try_get("revision").map_err(db::sql_error)?;
    if !(1..=9_007_199_254_740_991).contains(&revision) { return Err("Invalid stored edit revision".into()); }
    Ok((document,revision))
}

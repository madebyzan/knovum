use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

pub(in crate::spaces) async fn touch_updated_at(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    now: i64,
) -> Result<(), String> {
    sqlx::query("UPDATE spaces SET updated_at = $1 WHERE id = $2")
        .bind(now)
        .bind(space_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

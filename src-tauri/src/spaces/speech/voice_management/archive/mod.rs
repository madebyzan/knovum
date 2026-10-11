use sqlx::SqlitePool;

use crate::spaces::db;

pub(super) async fn update_state(
    pool: &SqlitePool,
    profile_id: &str,
    expected_updated_at: i64,
    is_archived: bool,
) -> Result<(), String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let archived_at = is_archived.then_some(now);
    let changed = sqlx::query(
        "UPDATE spaces_voice_profiles SET archived_at=?,updated_at=? WHERE id=? AND updated_at=?",
    )
    .bind(archived_at)
    .bind(now)
    .bind(profile_id)
    .bind(expected_updated_at)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        changed,
        "Voice profile changed before its archive state could be updated",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;

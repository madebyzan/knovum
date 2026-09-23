use sqlx::SqlitePool;

use super::row;
use crate::spaces::db;

pub(in crate::spaces::automation) async fn list_occurrences(
    pool: &SqlitePool,
    space_id: &str,
) -> Result<Vec<super::super::contract::AutomationOccurrence>, String> {
    let rows = sqlx::query(
        "SELECT o.* FROM spaces_automation_occurrences o
         JOIN spaces_automation_triggers t ON t.id=o.trigger_id
         WHERE t.space_id=$1 ORDER BY o.created_at DESC LIMIT 200",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.iter()
        .map(|record| {
            Ok(super::super::contract::AutomationOccurrence {
                id: row::get(record, "id")?,
                trigger_id: row::get(record, "trigger_id")?,
                scheduled_at: row::get(record, "scheduled_at")?,
                occurrence_key: row::get(record, "occurrence_key")?,
                run_id: row::get(record, "run_id")?,
                state: row::get(record, "state")?,
                detail: serde_json::from_str(&row::get::<String>(record, "detail")?)
                    .map_err(db::json_error)?,
                created_at: row::get(record, "created_at")?,
            })
        })
        .collect()
}

pub(in crate::spaces::automation) async fn advance_schedule(
    pool: &SqlitePool,
    id: &str,
    scheduled_at: i64,
    next_scheduled_at: i64,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_automation_triggers SET last_scheduled_at=$1,next_scheduled_at=$2,updated_at=$3 \
         WHERE id=$4 AND next_scheduled_at <= $1",
    )
    .bind(scheduled_at)
    .bind(next_scheduled_at)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

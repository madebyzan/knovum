use serde_json::Value;
use sqlx::Row;

use super::super::contract::AutomationTrigger;
use crate::spaces::db;

pub(super) fn trigger(row: &sqlx::sqlite::SqliteRow) -> Result<AutomationTrigger, String> {
    Ok(AutomationTrigger {
        id: get(row, "id")?,
        space_id: get(row, "space_id")?,
        name: get(row, "name")?,
        workflow_id: get(row, "workflow_id")?,
        workflow_version: get(row, "workflow_version")?,
        grant_id: get(row, "grant_id")?,
        source: serde_json::from_str(&get::<String>(row, "source_config")?)
            .map_err(db::json_error)?,
        input: serde_json::from_str(&get::<String>(row, "input")?).map_err(db::json_error)?,
        overlap_policy: enum_value(row, "overlap_policy")?,
        enabled: get(row, "enabled")?,
        valid_until: get(row, "valid_until")?,
        next_scheduled_at: get(row, "next_scheduled_at")?,
        last_scheduled_at: get(row, "last_scheduled_at")?,
        created_at: get(row, "created_at")?,
        updated_at: get(row, "updated_at")?,
    })
}

pub(super) fn get<T>(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<T, String>
where
    T: for<'row> sqlx::Decode<'row, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite>,
{
    row.try_get(field).map_err(db::sql_error)
}

fn enum_value<T: serde::de::DeserializeOwned>(
    row: &sqlx::sqlite::SqliteRow,
    field: &str,
) -> Result<T, String> {
    serde_json::from_value(Value::String(get(row, field)?)).map_err(db::json_error)
}

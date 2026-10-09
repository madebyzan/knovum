use serde::de::IntoDeserializer;
use sqlx::Row;

use super::RunObservation;
use crate::spaces::{authorization::RunScope, db};

pub(super) fn parse_summary(row: &sqlx::sqlite::SqliteRow) -> Result<RunObservation, String> {
    Ok(RunObservation {
        id: row.try_get("id").map_err(db::sql_error)?,
        scope: parse_scope(row)?,
        parent_run_id: row.try_get("parent_run_id").map_err(db::sql_error)?,
        retry_of_run_id: row.try_get("retry_of_run_id").map_err(db::sql_error)?,
        fork_of_run_id: row.try_get("fork_of_run_id").map_err(db::sql_error)?,
        kind: parse_enum(row, "kind")?,
        state: parse_enum(row, "state")?,
        state_version: row.try_get("state_version").map_err(db::sql_error)?,
        wait_reason: parse_optional_enum(row, "wait_reason")?,
        input_tokens: row.try_get("input_tokens").map_err(db::sql_error)?,
        output_tokens: row.try_get("output_tokens").map_err(db::sql_error)?,
        total_tokens: row.try_get("total_tokens").map_err(db::sql_error)?,
        step_attempts: row.try_get("step_attempts").map_err(db::sql_error)?,
        expanded_steps: row.try_get("expanded_steps").map_err(db::sql_error)?,
        output_bytes: row.try_get("output_bytes").map_err(db::sql_error)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
        completed_at: row.try_get("completed_at").map_err(db::sql_error)?,
    })
}

pub(super) fn parse_scope(row: &sqlx::sqlite::SqliteRow) -> Result<RunScope, String> {
    let kind: String = row.try_get("scope_kind").map_err(db::sql_error)?;
    let space_id = row.try_get("home_space_id").map_err(db::sql_error)?;
    RunScope::from_db(&kind, space_id)
}

pub(super) fn enum_text(value: impl serde::Serialize) -> Result<String, String> {
    serde_json::to_value(value)
        .map_err(db::json_error)?
        .as_str()
        .map(str::to_owned)
        .ok_or("Invalid enum".into())
}

pub(super) fn optional_enum_text(
    value: Option<impl serde::Serialize>,
) -> Result<Option<String>, String> {
    value.map(enum_text).transpose()
}

fn parse_enum<T: serde::de::DeserializeOwned>(
    row: &sqlx::sqlite::SqliteRow,
    field: &str,
) -> Result<T, String> {
    let value: String = row.try_get(field).map_err(db::sql_error)?;
    T::deserialize(value.into_deserializer()).map_err(db::json_error)
}

fn parse_optional_enum<T: serde::de::DeserializeOwned>(
    row: &sqlx::sqlite::SqliteRow,
    field: &str,
) -> Result<Option<T>, String> {
    let value: Option<String> = row.try_get(field).map_err(db::sql_error)?;
    value
        .map(|item| T::deserialize(item.into_deserializer()).map_err(db::json_error))
        .transpose()
}

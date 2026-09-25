use sqlx::{Row, Sqlite, Transaction};

use super::{AuthorizationContext, Principal};
use crate::spaces::db;

pub(crate) async fn create_grant(
    tx: &mut Transaction<'_, Sqlite>,
    context: &AuthorizationContext,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_authorization_grants(
          id,principal_kind,principal_id,home_space_id,read_space_ids,write_space_ids,
          action_ids,valid_until,revoked_at,created_at
        ) VALUES($1,$2,$3,$4,$5,$6,$7,$8,NULL,$9)",
    )
    .bind(&context.grant_id)
    .bind(context.principal.kind.as_str())
    .bind(&context.principal.id)
    .bind(&context.home_space_id)
    .bind(serde_json::to_string(&context.read_space_ids).map_err(db::json_error)?)
    .bind(serde_json::to_string(&context.write_space_ids).map_err(db::json_error)?)
    .bind(serde_json::to_string(&context.action_ids).map_err(db::json_error)?)
    .bind(context.valid_until)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(crate) async fn load_grant(
    tx: &mut Transaction<'_, Sqlite>,
    grant_id: &str,
) -> Result<AuthorizationContext, String> {
    let row = sqlx::query(
        "SELECT principal_kind,principal_id,home_space_id,read_space_ids,write_space_ids,
         action_ids,valid_until,revoked_at
         FROM spaces_authorization_grants WHERE id=$1",
    )
    .bind(grant_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("permission-denied: authorization grant is unavailable")?;
    if row
        .try_get::<Option<i64>, _>("revoked_at")
        .map_err(db::sql_error)?
        .is_some()
    {
        return Err("permission-denied: authorization grant was revoked".into());
    }
    let kind = serde_json::from_value(serde_json::Value::String(
        row.try_get("principal_kind").map_err(db::sql_error)?,
    ))
    .map_err(db::json_error)?;
    Ok(AuthorizationContext {
        principal: Principal {
            kind,
            id: row.try_get("principal_id").map_err(db::sql_error)?,
        },
        grant_id: grant_id.to_owned(),
        home_space_id: row.try_get("home_space_id").map_err(db::sql_error)?,
        read_space_ids: parse_array(&row, "read_space_ids")?,
        write_space_ids: parse_array(&row, "write_space_ids")?,
        action_ids: parse_array(&row, "action_ids")?,
        valid_until: row.try_get("valid_until").map_err(db::sql_error)?,
    })
}

fn parse_array(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<Vec<String>, String> {
    let value: String = row.try_get(field).map_err(db::sql_error)?;
    serde_json::from_str(&value).map_err(db::json_error)
}

use serde_json::Value;
use sqlx::{Row, SqlitePool};

use super::{
    contract::{ExternalToolBinding, SaveExternalToolBindingRequest},
    fingerprint,
};
use crate::spaces::db;

pub(super) async fn save(
    pool: &SqlitePool,
    request: SaveExternalToolBindingRequest,
) -> Result<ExternalToolBinding, String> {
    validate_request(&request)?;
    let discovered = super::client::discover(
        &request.endpoint,
        request.secret_key.as_deref(),
        &request.tool_name,
    )
    .await?;
    let fingerprint = fingerprint::calculate(
        &discovered.server_identity,
        &request.tool_name,
        &discovered.input_schema,
        discovered.output_schema.as_ref(),
    )?;
    let now = crate::clock::unix_time_millis()?;
    sqlx::query(
        "INSERT INTO spaces_external_tool_bindings(
          id,space_id,name,endpoint,secret_key,server_identity,tool_name,input_schema,
          output_schema,schema_fingerprint,effect,enabled,reviewed_at,created_at,updated_at
        ) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$13,$13)
        ON CONFLICT(id) DO UPDATE SET name=excluded.name,endpoint=excluded.endpoint,
          secret_key=excluded.secret_key,server_identity=excluded.server_identity,
          tool_name=excluded.tool_name,input_schema=excluded.input_schema,
          output_schema=excluded.output_schema,schema_fingerprint=excluded.schema_fingerprint,
          effect=excluded.effect,enabled=excluded.enabled,reviewed_at=excluded.reviewed_at,
          updated_at=excluded.updated_at",
    )
    .bind(&request.id)
    .bind(&request.space_id)
    .bind(request.name.trim())
    .bind(request.endpoint.trim())
    .bind(
        request
            .secret_key
            .as_deref()
            .filter(|value| !value.trim().is_empty()),
    )
    .bind(serde_json::to_string(&discovered.server_identity).map_err(db::json_error)?)
    .bind(request.tool_name.trim())
    .bind(serde_json::to_string(&discovered.input_schema).map_err(db::json_error)?)
    .bind(
        discovered
            .output_schema
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(db::json_error)?,
    )
    .bind(fingerprint)
    .bind(request.effect.as_str())
    .bind(request.enabled)
    .bind(now)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    load(pool, &request.id).await
}

pub(super) async fn list(
    pool: &SqlitePool,
    space_id: &str,
) -> Result<Vec<ExternalToolBinding>, String> {
    let rows = sqlx::query(
        "SELECT * FROM spaces_external_tool_bindings WHERE space_id=$1 ORDER BY updated_at DESC",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.iter().map(parse).collect()
}

pub(in crate::spaces::workflows) async fn load(
    pool: &SqlitePool,
    binding_id: &str,
) -> Result<ExternalToolBinding, String> {
    let row = sqlx::query("SELECT * FROM spaces_external_tool_bindings WHERE id=$1")
        .bind(binding_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?
        .ok_or("external-tool-unavailable: binding is unavailable")?;
    parse(&row)
}

pub(super) async fn delete(pool: &SqlitePool, binding_id: &str) -> Result<(), String> {
    let references: i64 = sqlx::query_scalar(
        "SELECT
          (SELECT COUNT(*) FROM spaces_workflow_drafts d,json_each(d.definition,'$.nodes') n
           WHERE json_extract(n.value,'$.kind')='external_tool' AND json_extract(n.value,'$.bindingId')=$1)
          +
          (SELECT COUNT(*) FROM spaces_workflow_versions v,json_each(v.definition,'$.nodes') n
           WHERE json_extract(n.value,'$.kind')='external_tool' AND json_extract(n.value,'$.bindingId')=$1)",
    ).bind(binding_id).fetch_one(pool).await.map_err(db::sql_error)?;
    if references != 0 {
        return Err("external-tool-in-use: remove the binding from Workflow drafts and versions before deletion".into());
    }
    let result = sqlx::query("DELETE FROM spaces_external_tool_bindings WHERE id=$1")
        .bind(binding_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    db::require_one(result, "external-tool-unavailable: binding is unavailable")
}

pub(super) async fn disable(pool: &SqlitePool, binding_id: &str) -> Result<(), String> {
    let result = sqlx::query(
        "UPDATE spaces_external_tool_bindings SET enabled=0,updated_at=$1 WHERE id=$2 AND enabled=1",
    ).bind(crate::clock::unix_time_millis()?).bind(binding_id)
        .execute(pool).await.map_err(db::sql_error)?;
    if result.rows_affected() == 0 {
        let exists: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM spaces_external_tool_bindings WHERE id=$1")
                .bind(binding_id)
                .fetch_optional(pool)
                .await
                .map_err(db::sql_error)?;
        if exists.is_none() {
            return Err("external-tool-unavailable: binding is unavailable".into());
        }
    }
    Ok(())
}

fn validate_request(request: &SaveExternalToolBindingRequest) -> Result<(), String> {
    if request.id.trim().is_empty()
        || request.space_id.trim().is_empty()
        || request.name.trim().is_empty()
        || request.endpoint.trim().is_empty()
        || request.tool_name.trim().is_empty()
    {
        return Err("external-tool-binding-invalid: all identity fields are required".into());
    }
    Ok(())
}

fn parse(row: &sqlx::sqlite::SqliteRow) -> Result<ExternalToolBinding, String> {
    Ok(ExternalToolBinding {
        id: get(row, "id")?,
        space_id: get(row, "space_id")?,
        name: get(row, "name")?,
        endpoint: get(row, "endpoint")?,
        secret_key: get(row, "secret_key")?,
        server_identity: json(row, "server_identity")?,
        tool_name: get(row, "tool_name")?,
        input_schema: json(row, "input_schema")?,
        output_schema: get::<Option<String>>(row, "output_schema")?
            .map(|value| serde_json::from_str(&value).map_err(db::json_error))
            .transpose()?,
        schema_fingerprint: get(row, "schema_fingerprint")?,
        effect: serde_json::from_value(Value::String(get(row, "effect")?))
            .map_err(db::json_error)?,
        enabled: get(row, "enabled")?,
        reviewed_at: get(row, "reviewed_at")?,
        created_at: get(row, "created_at")?,
        updated_at: get(row, "updated_at")?,
    })
}

fn json(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<Value, String> {
    serde_json::from_str(&get::<String>(row, field)?).map_err(db::json_error)
}

fn get<T>(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<T, String>
where
    T: for<'row> sqlx::Decode<'row, sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite>,
{
    row.try_get(field).map_err(db::sql_error)
}

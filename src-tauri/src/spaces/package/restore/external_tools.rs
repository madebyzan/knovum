use sqlx::{Sqlite, Transaction};

use super::contracts::RestorePackageRequest;
use crate::spaces::db;

pub(super) async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.external_tool_bindings {
        sqlx::query("INSERT INTO spaces_external_tool_bindings(id,space_id,name,endpoint,secret_key,server_identity,tool_name,input_schema,output_schema,schema_fingerprint,effect,enabled,reviewed_at,created_at,updated_at) VALUES($1,$2,$3,$4,NULL,$5,$6,$7,$8,$9,$10,0,$11,$12,$13)")
            .bind(&row.id).bind(&request.space.id).bind(&row.name).bind(&row.endpoint)
            .bind(json(&row.server_identity)?).bind(&row.tool_name).bind(json(&row.input_schema)?)
            .bind(optional_json(row.output_schema.as_ref())?).bind(&row.schema_fingerprint)
            .bind(&row.effect).bind(row.reviewed_at).bind(row.created_at).bind(row.updated_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

fn json(value: &serde_json::Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(db::json_error)
}
fn optional_json(value: Option<&serde_json::Value>) -> Result<Option<String>, String> {
    value.map(json).transpose()
}

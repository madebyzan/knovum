mod completion;
mod source;

use serde::Deserialize;
use serde_json::Value;
use tauri::AppHandle;

use super::{failure, inputs, CompletedDerivationOutput};
use crate::spaces::{db, knowledge::contracts};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartImageCutoutRequest {
    id: String,
    space_id: String,
    source_node_id: String,
    output_node_id: String,
    source_revision: String,
    source_edit: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteImageCutoutRequest {
    id: String,
    source_edit: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FailImageCutoutRequest {
    id: String,
    error: String,
}

pub async fn start(app: &AppHandle, request: StartImageCutoutRequest) -> Result<(), String> {
    validate_request(&request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    contracts::validate_same_space_node(
        &mut tx,
        &request.space_id,
        &request.source_node_id,
        &request.output_node_id,
    )
    .await?;
    let source_data: Option<String> = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='image' AND deleted_at IS NULL",
    )
    .bind(&request.source_node_id)
    .bind(&request.space_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let source_data: Value = serde_json::from_str(&source_data.ok_or("Image source is unavailable")?)
        .map_err(db::json_error)?;
    if source_data.get("sourceRevision").and_then(Value::as_str)
        != Some(request.source_revision.as_str())
        || source_data.get("imageEdit").unwrap_or(&Value::Null) != &request.source_edit
    {
        return Err("Image source changed before background removal started".into());
    }
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,'remove-image-background',1,'u2netp-tract:1','running',$4,$4)")
        .bind(&request.id)
        .bind(&request.space_id)
        .bind(&request.output_node_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    inputs::insert(
        &mut tx,
        &request.id,
        &[inputs::DerivationInput {
            node_id: &request.source_node_id,
            role: "primary",
            source_revision: &request.source_revision,
        }],
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)
}

pub async fn complete(
    app: AppHandle,
    request: CompleteImageCutoutRequest,
) -> Result<CompletedDerivationOutput, String> {
    completion::complete(app, request).await
}

pub async fn fail(app: &AppHandle, request: FailImageCutoutRequest) -> Result<(), String> {
    failure::fail_message(app, &request.id, &request.error).await
}

fn validate_request(request: &StartImageCutoutRequest) -> Result<(), String> {
    if request.id.trim().is_empty() || request.space_id.trim().is_empty() {
        return Err("Derivation identity is required".into());
    }
    if request.source_node_id == request.output_node_id || request.source_revision.trim().is_empty()
    {
        return Err("Background removal source is invalid".into());
    }
    if !request.source_edit.is_null() && !request.source_edit.is_object() {
        return Err("Background removal image edit is invalid".into());
    }
    Ok(())
}

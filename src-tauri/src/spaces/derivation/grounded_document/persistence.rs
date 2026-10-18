use tauri::AppHandle;

use super::GenerateGroundedDocumentRequest;
use crate::spaces::db;
use crate::spaces::derivation::inputs;
use crate::spaces::derivation::source_inputs;

pub async fn start(
    app: &AppHandle,
    request: &GenerateGroundedDocumentRequest,
    authority: &super::super::action_execution::ExecutionAuthority,
) -> Result<(), String> {
    validate_request(request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    source_inputs::pin_for_output(
        &mut tx,
        &request.space_id,
        &request.output_node_id,
        &request.inputs,
    )
    .await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    sqlx::query(
        "INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,(SELECT json_extract(input_snapshot,'$.actionId') FROM spaces_runs WHERE id=$1 AND kind='action'),1,'knovum-grounded-ai:1','running',$4,$4)",
    )
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
        &source_inputs::records(&request.inputs),
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)
}

pub async fn fail(app: &AppHandle, derivation_id: &str, message: &str) -> Result<(), String> {
    crate::spaces::derivation::failure::fail_message(app, derivation_id, message).await
}

fn validate_request(request: &GenerateGroundedDocumentRequest) -> Result<(), String> {
    if request.id.trim().is_empty()
        || request.space_id.trim().is_empty()
        || request.output_node_id.trim().is_empty()
        || request.purpose.trim().is_empty()
    {
        return Err("Grounded document request is invalid".into());
    }
    source_inputs::require_single_primary(&request.inputs)
}

use serde_json::Value;

use super::{client, fingerprint, repository, ExternalToolBinding, SaveExternalToolBindingRequest};

#[tauri::command]
pub async fn save_external_tool_binding(
    app: tauri::AppHandle,
    request: SaveExternalToolBindingRequest,
) -> Result<ExternalToolBinding, String> {
    repository::save(crate::spaces::db::pool(&app).await?, request).await
}

#[tauri::command]
pub async fn list_external_tool_bindings(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<ExternalToolBinding>, String> {
    repository::list(crate::spaces::db::pool(&app).await?, &space_id).await
}

#[tauri::command]
pub async fn delete_external_tool_binding(
    app: tauri::AppHandle,
    binding_id: String,
) -> Result<(), String> {
    repository::delete(crate::spaces::db::pool(&app).await?, &binding_id).await
}

#[tauri::command]
pub async fn disable_external_tool_binding(
    app: tauri::AppHandle,
    binding_id: String,
) -> Result<(), String> {
    repository::disable(crate::spaces::db::pool(&app).await?, &binding_id).await
}

#[tauri::command]
pub async fn test_external_tool_connection(
    request: SaveExternalToolBindingRequest,
) -> Result<Value, String> {
    let discovered = client::discover(
        &request.endpoint,
        request.secret_key.as_deref(),
        &request.tool_name,
    )
    .await?;
    let schema_fingerprint = fingerprint::calculate(
        &discovered.server_identity,
        &request.tool_name,
        &discovered.input_schema,
        discovered.output_schema.as_ref(),
    )?;
    Ok(serde_json::json!({
        "serverIdentity": discovered.server_identity,
        "toolName": request.tool_name,
        "inputSchema": discovered.input_schema,
        "outputSchema": discovered.output_schema,
        "schemaFingerprint": schema_fingerprint,
    }))
}

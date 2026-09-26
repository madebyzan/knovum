use crate::command_error::CommandError;
pub(crate) mod contract;
mod export;
pub(crate) mod import;
mod validation;

use contract::JsonCanvasDocumentV1;

#[tauri::command]
pub(crate) async fn export_json_canvas_file(
    app: tauri::AppHandle,
    request: export::ExportRequest,
) -> Result<(), CommandError> {
    Ok(
        (tauri::async_runtime::spawn_blocking(move || export::write(&app, request))
            .await
            .map_err(|error| error.to_string())?)?,
    )
}

#[tauri::command]
pub(crate) fn parse_json_canvas(source: String) -> Result<JsonCanvasDocumentV1, CommandError> {
    let document = serde_json::from_str(&source).map_err(|error| error.to_string())?;
    validation::validate(&document)?;
    Ok(document)
}

#[tauri::command]
pub(crate) fn serialize_json_canvas(
    document: JsonCanvasDocumentV1,
) -> Result<String, CommandError> {
    validation::validate(&document)?;
    Ok((serde_json::to_string_pretty(&document).map_err(|error| error.to_string()))?)
}

#[tauri::command]
pub(crate) async fn import_json_canvas_nodes<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    request: import::ImportJsonCanvasNodesRequest,
) -> Result<import::ImportJsonCanvasResult, CommandError> {
    Ok((import::import_nodes(app, request).await)?)
}

#[cfg(test)]
mod tests;

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde_json::Value;
use tauri::AppHandle;
use uuid::Uuid;

use super::{access, enrichment};
use crate::foundation::app_error::AppError;

const MAX_PDF_ASSET_BYTES: usize = 64 * 1024 * 1024;
const PDF_HEADER: &[u8] = b"%PDF-";

fn ensure_pdf_node(payload: &Value) -> Result<(), AppError> {
    let is_pdf = payload.get("kind").and_then(Value::as_str) == Some("file")
        && payload
            .get("fileName")
            .and_then(Value::as_str)
            .is_some_and(|name| name.to_ascii_lowercase().ends_with(".pdf"));
    if is_pdf {
        Ok(())
    } else {
        Err(AppError::new(
            "invalid_argument",
            "Only PDF Space assets can be saved through PDF markup",
        ))
    }
}

fn validate_pdf_bytes(bytes: &[u8]) -> Result<(), AppError> {
    if bytes.is_empty() || bytes.len() > MAX_PDF_ASSET_BYTES {
        return Err(AppError::new(
            "invalid_argument",
            "PDF document exceeds the 64 MB in-app save limit",
        ));
    }
    if !bytes.starts_with(PDF_HEADER) {
        return Err(AppError::new(
            "invalid_argument",
            "Saved PDF payload must begin with a valid %PDF- header",
        ));
    }
    Ok(())
}

fn atomic_replace_file(target: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = target
        .parent()
        .ok_or_else(|| "Target asset path has no parent directory".to_owned())?;
    let temp = parent.join(format!(".knovum-edit-{}.tmp", Uuid::new_v4()));
    let write_result = (|| -> Result<(), String> {
        let mut file = fs::File::create(&temp).map_err(|error| error.to_string())?;
        file.write_all(bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    if let Err(error) = fs::rename(&temp, target) {
        let _ = fs::remove_file(&temp);
        return Err(error.to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn save_editable_asset(
    app: AppHandle,
    space_id: String,
    node_id: String,
    bytes: Vec<u8>,
) -> Result<Option<Value>, AppError> {
    let payload = enrichment::node_payload::load_node_content(&app, &space_id, &node_id).await?;
    ensure_pdf_node(&payload)?;
    validate_pdf_bytes(&bytes)?;
    let source = enrichment::node_payload::required_string(&payload, "assetPath")?;
    let write_app = app.clone();
    crate::foundation::blocking::run(move || {
        let canonical = access::authorize_spaces_asset(&write_app, &PathBuf::from(source))?;
        atomic_replace_file(&canonical, &bytes)
    })
    .await?;
    enrichment::refresh::refresh_node(&app, space_id, node_id).await
}

#[cfg(test)]
#[path = "tests/edit.rs"]
mod tests;

use std::time::Duration;
use tauri::AppHandle;

use crate::{app_error::AppError, local_resources::preparation::locks};

use super::DiarizationModelImportResult;

#[tauri::command]
pub async fn import_diarization_model(
    app: AppHandle,
    source_path: String,
    accepted_terms: bool,
) -> Result<DiarizationModelImportResult, AppError> {
    if !accepted_terms {
        return Err(AppError::invalid(
            "Diarization import requires the user's declaration that official model terms were accepted",
        ));
    }
    if source_path.trim().is_empty() {
        return Err(AppError::invalid(
            "Diarization import directory is required",
        ));
    }
    let _installation = locks::acquire_diarization_installation().await;
    Ok(crate::blocking::run(move || super::import(&app, &source_path)).await?)
}

#[tauri::command]
pub async fn remove_diarization_model(app: AppHandle) -> Result<(), AppError> {
    let _installation = tokio::time::timeout(
        Duration::from_secs(2),
        locks::acquire_diarization_installation(),
    )
    .await
    .map_err(|_| {
        AppError::new(
            "resource_in_use",
            "Diarization model is currently used by an active job",
        )
    })?;
    Ok(crate::blocking::run(move || super::remove(&app)).await?)
}

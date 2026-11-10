use std::time::Duration;

use tauri::AppHandle;

use super::{filesystem, model_catalog::LocalModelId, preparation::locks};
use crate::foundation::app_error::AppError;

#[tauri::command]
pub async fn remove_model(app: AppHandle, model: LocalModelId) -> Result<(), AppError> {
    if model == LocalModelId::Qwen3Embedding {
        return Err(AppError::invalid(
            "Essential local models are managed by the application",
        ));
    }
    let usage = tokio::time::timeout(
        Duration::from_secs(2),
        locks::resource_lock(model.snapshot().ok_or("Model snapshot is unavailable")?)?
            .write_owned(),
    )
    .await
    .map_err(|_| in_use())?;
    if model.is_transcription() && super::model_selection::transcription(&app)? == model {
        return Err(AppError::invalid(
            "Select another transcription model before removing the active model",
        ));
    }
    crate::foundation::blocking::run(move || {
        let snapshot = model.snapshot().ok_or("Model snapshot is unavailable")?;
        let target = super::models::snapshot_directory(&app, snapshot)?;
        let parent = target
            .parent()
            .ok_or("Model snapshot root is unavailable")?;
        if !parent.exists() {
            return Ok(());
        }
        filesystem::verify_managed_root(
            &app,
            parent
                .parent()
                .ok_or("Model resource root is unavailable")?,
        )?;
        filesystem::verify_regular_directory(parent)?;
        filesystem::remove_owned_directory(&target, parent)
    })
    .await?;
    drop(usage);
    Ok(())
}

fn in_use() -> AppError {
    AppError::new("resource_in_use", "This model is used by an active job")
}

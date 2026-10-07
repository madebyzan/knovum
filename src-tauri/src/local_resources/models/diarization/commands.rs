use std::time::Duration;
use tauri::AppHandle;

use crate::{app_error::AppError, local_resources::preparation::locks};

#[tauri::command]
pub async fn set_diarization_access_token(token: Option<String>) -> Result<(), AppError> {
    match token {
        Some(token) => {
            let token = token.trim();
            if token.is_empty()
                || token.len() > 4096
                || reqwest::header::HeaderValue::from_str(&format!("Bearer {token}")).is_err()
            {
                return Err(AppError::invalid(
                    "A valid Hugging Face read access token is required",
                ));
            }
            crate::secrets::write_secret(super::ACCESS_TOKEN_KEY, token).await?;
        }
        None => crate::secrets::remove_secret(super::ACCESS_TOKEN_KEY).await?,
    }
    Ok(())
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

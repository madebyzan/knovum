use crate::command_error::CommandError;
use tauri::{plugin::TauriPlugin, AppHandle, Wry};

use crate::local_resources::{models, models::catalog::ModelKind};

#[tauri::command]
pub async fn remove_model(app: AppHandle, model: ModelKind) -> Result<(), CommandError> {
    if matches!(
        model,
        ModelKind::TextEmbeddingConfig
            | ModelKind::TextEmbeddingTokenizer
            | ModelKind::TextEmbeddingWeights
    ) {
        crate::ai::capabilities::embedding::unload()?;
    }
    Ok(
        (crate::blocking::run(move || {
            models::uninstall(&app, models::catalog::descriptor(model))
        })
        .await)?,
    )
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("local-resources")
        .invoke_handler(tauri::generate_handler![
            crate::local_resources::get_status,
            crate::local_resources::models::commands::install_models,
            crate::local_resources::models::commands::cancel_model_install,
            crate::local_resources::models::commands::list_model_installs,
            remove_model,
        ])
        .build()
}

use crate::command_error::CommandError;
use tauri::{plugin::TauriPlugin, AppHandle, Wry};

use crate::local_resources::{models::catalog::ModelKind, preparation};

#[tauri::command]
pub async fn remove_model(app: AppHandle, model: ModelKind) -> Result<(), CommandError> {
    let removal = preparation::acquire_model_removal(model)?;
    if matches!(
        model,
        ModelKind::TextEmbeddingConfig
            | ModelKind::TextEmbeddingTokenizer
            | ModelKind::TextEmbeddingWeights
    ) {
        crate::ai::capabilities::embedding::unload()?;
    }
    Ok((crate::blocking::run(move || removal.remove(&app)).await)?)
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("local-resources")
        .invoke_handler(tauri::generate_handler![
            crate::local_resources::get_status,
            crate::local_resources::preparation::commands::prepare_resources,
            crate::local_resources::preparation::commands::cancel_resource_preparation,
            crate::local_resources::preparation::commands::list_resource_preparations,
            remove_model,
        ])
        .build()
}

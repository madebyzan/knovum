use tauri::{ipc::Channel, plugin::TauriPlugin, AppHandle, Wry};

use crate::{
    foundation::app_error::AppError,
    local_resources::{
        model_catalog::{self, LocalModelAvailability, LocalModelId},
        preparation::contracts::{ResourcePreparationProgress, ResourcePreparationTarget},
    },
};

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("local-resources")
        .invoke_handler(tauri::generate_handler![
            crate::local_resources::get_status,
            get_model_catalog,
            crate::local_resources::model_commands::remove_model,
            select_local_model,
            prepare_resources,
            crate::local_resources::preparation::commands::cancel_resource_preparation,
            crate::local_resources::preparation::commands::list_resource_preparations,
        ])
        .build()
}

#[tauri::command]
async fn get_model_catalog(app: tauri::AppHandle) -> Result<Vec<LocalModelAvailability>, AppError> {
    Ok(crate::foundation::blocking::run(move || model_catalog::list(&app)).await?)
}

#[tauri::command]
async fn select_local_model(app: AppHandle, model: LocalModelId) -> Result<(), AppError> {
    crate::local_resources::model_selection::select_local_model(app, model).await
}

#[tauri::command]
async fn prepare_resources(
    app: AppHandle,
    target: ResourcePreparationTarget,
    job_id: String,
    title: String,
    on_progress: Channel<ResourcePreparationProgress>,
) -> Result<(), AppError> {
    crate::local_resources::preparation::commands::prepare_resources(
        app,
        target,
        job_id,
        title,
        on_progress,
    )
    .await
}

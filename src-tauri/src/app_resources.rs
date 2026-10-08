use tauri::{ipc::Channel, plugin::TauriPlugin, AppHandle, Wry};

use crate::{
    app_error::AppError,
    local_resources::{
        model_catalog::{self, LocalModelEntry, LocalModelId},
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
async fn get_model_catalog(app: tauri::AppHandle) -> Result<Vec<LocalModelEntry>, AppError> {
    Ok(crate::blocking::run(move || {
        model_catalog::list(&app, |model| {
            crate::ai::local_runtime::device_profiles::voice::eligible(model.voxcpm2_spec()?)
        })
    })
    .await?)
}

#[tauri::command]
async fn select_local_model(app: AppHandle, model: LocalModelId) -> Result<(), AppError> {
    ensure_model_available(model)?;
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
    match &target {
        ResourcePreparationTarget::Model { model } => ensure_model_available(*model)?,
        ResourcePreparationTarget::VoiceDesign => {
            ensure_model_available(LocalModelId::VoxCpm2)?;
        }
        _ => {}
    }
    crate::local_resources::preparation::commands::prepare_resources(
        app,
        target,
        job_id,
        title,
        on_progress,
    )
    .await
}

fn ensure_model_available(model: LocalModelId) -> Result<(), AppError> {
    if matches!(model, LocalModelId::VoxCpm2 | LocalModelId::VoxCpm2Q8)
        && !crate::ai::local_runtime::device_profiles::voice::eligible(model.voxcpm2_spec()?)?
    {
        return Err(AppError::new(
            "model_unavailable",
            "This model does not meet this computer's hardware requirements",
        ));
    }
    Ok(())
}

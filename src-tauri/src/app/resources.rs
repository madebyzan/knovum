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
    Ok(
        crate::foundation::blocking::run(move || model_catalog::list(&app, voice_memory_advisory))
            .await?,
    )
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

/// How much memory the model was qualified against, reported only when this machine has less.
/// Downloading and selecting a model are never refused for it: disk space is what a download
/// spends, and whether a run may start is admission's decision against free memory at that moment.
fn voice_memory_advisory(model: LocalModelId) -> Result<Option<u64>, String> {
    if !matches!(model, LocalModelId::VoxCpm2 | LocalModelId::VoxCpm2Q8) {
        return Ok(None);
    }
    let recommended = crate::ai::local_runtime::device_profiles::voice::recommended_memory_bytes(
        model.voxcpm2_spec()?,
    )?;
    Ok(
        (crate::local_resources::system_memory::physical_memory()? < recommended)
            .then_some(recommended),
    )
}

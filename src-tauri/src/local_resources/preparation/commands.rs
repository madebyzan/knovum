use tauri::{ipc::Channel, AppHandle};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use super::{
    contracts::{
        ResourcePreparationProgress, ResourcePreparationSnapshot, ResourcePreparationTarget,
    },
    requests,
};
use crate::app_error::AppError;

#[tauri::command]
pub async fn prepare_resources(
    app: AppHandle,
    target: ResourcePreparationTarget,
    job_id: String,
    title: String,
    on_progress: Channel<ResourcePreparationProgress>,
) -> Result<(), AppError> {
    if matches!(
        &target,
        ResourcePreparationTarget::Transcription | ResourcePreparationTarget::SpeechAlignment
    ) && target.models().is_none()
    {
        return Err(
            "unsupported_device: compiled speech resources are not available on this platform"
                .into(),
        );
    }
    let request = requests::begin(&job_id, &title, target.clone())?;
    let (finished_tx, finished_rx) = oneshot::channel();
    tauri::async_runtime::spawn(async move {
        let effective_target = match target {
            ResourcePreparationTarget::Model { model } if !model.is_transcription() => {
                model.preparation()
            }
            other => other,
        };
        let preparation_outcome = match effective_target {
            target @ (ResourcePreparationTarget::Transcription
            | ResourcePreparationTarget::SpeechSynthesis
            | ResourcePreparationTarget::TextEmbedding
            | ResourcePreparationTarget::ImageSegmentation
            | ResourcePreparationTarget::SpeechAlignment
            | ResourcePreparationTarget::SpeechSeparation
            | ResourcePreparationTarget::SpeechDiarization
            | ResourcePreparationTarget::VoiceDesign
            | ResourcePreparationTarget::Model { .. }) => {
                let kinds = target
                    .models()
                    .expect("model preparation targets own a fixed model profile");
                super::prepare_models_for_request(
                    &app,
                    kinds,
                    request,
                    &CancellationToken::new(),
                    &mut |progress| observe(&on_progress, progress),
                )
                .await
                .map(|_| ())
            }
            ResourcePreparationTarget::BuiltinAgent => {
                super::prepare_agent_for_request(
                    &app,
                    request,
                    &CancellationToken::new(),
                    &mut |progress| observe(&on_progress, progress),
                )
                .await
            }
            ResourcePreparationTarget::MediaImportTools => {
                crate::local_resources::tools::prepare_media_import_tools_for_request(
                    &app,
                    request,
                    &CancellationToken::new(),
                    &mut |progress| observe(&on_progress, progress),
                )
                .await
                .map(|_| ())
            }
        };
        let _ = finished_tx.send(preparation_outcome);
    });
    let outcome = finished_rx
        .await
        .map_err(|_| "Resource preparation task ended unexpectedly")?;
    outcome.map_err(|message| {
        if message == super::RESOURCE_PREPARATION_CANCELLED {
            AppError::new("local_resource_preparation_cancelled", message)
        } else {
            message.into()
        }
    })?;
    Ok(())
}

#[tauri::command]
pub fn cancel_resource_preparation(job_id: String) -> Result<(), AppError> {
    requests::cancel(&job_id)?;
    Ok(())
}

#[tauri::command]
pub fn list_resource_preparations() -> Result<Vec<ResourcePreparationSnapshot>, AppError> {
    Ok(requests::list()?)
}

fn observe(channel: &Channel<ResourcePreparationProgress>, progress: ResourcePreparationProgress) {
    if let Err(error) = channel.send(progress) {
        log::debug!("resource preparation observer disconnected: {error}");
    }
}

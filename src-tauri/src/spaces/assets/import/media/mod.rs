use crate::app_error::AppError;
mod acquisition;
mod contract;
mod jobs;
mod publish;

use crate::media::acquisition::contracts::AcquisitionProgress;
pub(crate) use contract::ImportedMedia;

pub(crate) fn cleanup_stale_workspaces(app: &tauri::AppHandle) -> Result<(), String> {
    acquisition::cleanup_stale(app)
}

#[tauri::command]
pub async fn import_media_from_url(
    app: tauri::AppHandle,
    job_id: String,
    resource_title: String,
    url: String,
    on_progress: tauri::ipc::Channel<AcquisitionProgress>,
) -> Result<Option<ImportedMedia>, AppError> {
    acquisition::validate_source_url(&url)?;
    acquisition::validate_job_id(&job_id)?;
    let (cancellation, _lease) = jobs::acquire(job_id.clone())?;
    let resource_job_id = format!("resource:{job_id}");
    let tools = crate::media::acquisition::tools::prepare_required_for_job(
        &app,
        &resource_job_id,
        &resource_title,
        &cancellation,
        &mut |progress| {
            observe(
                &on_progress,
                AcquisitionProgress {
                    stage: "preparing",
                    downloaded_bytes: progress.downloaded,
                    total_bytes: progress.total,
                    processed_ms: None,
                    duration_ms: None,
                },
            );
        },
    )
    .await
    .map_err(media_preparation_error)?;
    observe(
        &on_progress,
        AcquisitionProgress {
            stage: "resolving",
            downloaded_bytes: 0,
            total_bytes: None,
            processed_ms: None,
            duration_ms: None,
        },
    );
    let execution_id = job_id;
    crate::blocking::run(move || {
        let _job = _lease;
        acquisition::run(&app, &execution_id, url, tools, &on_progress, &cancellation)
    })
    .await
    .map_err(media_acquisition_error)
}

fn media_preparation_error(error: String) -> AppError {
    if error == crate::local_resources::preparation::RESOURCE_PREPARATION_CANCELLED {
        return AppError::new(
            "media_acquisition_cancelled",
            format!(
                "{}: Media import was cancelled",
                crate::media::acquisition::yt_dlp::ACQUISITION_CANCELLED
            ),
        );
    }
    error.into()
}

fn media_acquisition_error(error: String) -> AppError {
    if error == crate::media::acquisition::yt_dlp::ACQUISITION_CANCELLED {
        AppError::new("media_acquisition_cancelled", error)
    } else {
        error.into()
    }
}

fn observe(channel: &tauri::ipc::Channel<AcquisitionProgress>, progress: AcquisitionProgress) {
    if let Err(error) = channel.send(progress) {
        log::debug!("media import progress observer disconnected: {error}");
    }
}

#[tauri::command]
pub async fn cancel_media_import(job_id: String) -> Result<(), AppError> {
    Ok((jobs::cancel(&job_id))?)
}

#[cfg(test)]
mod tests {
    use super::{media_acquisition_error, media_preparation_error};

    #[test]
    fn media_cancellation_has_an_explicit_command_code() {
        let acquisition = media_acquisition_error(
            crate::media::acquisition::yt_dlp::ACQUISITION_CANCELLED.to_owned(),
        );
        let preparation = media_preparation_error(
            crate::local_resources::preparation::RESOURCE_PREPARATION_CANCELLED.to_owned(),
        );
        assert_eq!(acquisition.code, "media_acquisition_cancelled");
        assert_eq!(preparation.code, "media_acquisition_cancelled");
        assert!(preparation.message.contains("Media import was cancelled"));
    }
}

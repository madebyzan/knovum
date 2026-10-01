use std::{sync::atomic::{AtomicBool, Ordering}, time::{Duration, Instant}};

use tauri::{AppHandle, State};
use tauri_plugin_updater::Error;
use tokio_util::sync::CancellationToken;

use crate::app_error::AppError;
use super::{UpdateFailure, UpdateMetadata, UpdateService, UpdateSnapshot, UpdateState};

// The official updater buffers the archive in memory before verifying it.
const MAX_DOWNLOAD_BYTES: u64 = 1024 * 1024 * 1024;

#[tauri::command]
pub fn cancel_update_download(service: State<'_, UpdateService>) -> Result<(), AppError> {
    if let Some(cancellation) = &service.lock()?.cancellation {
        cancellation.cancel();
    }
    Ok(())
}

#[tauri::command]
pub async fn download_update(app: AppHandle, service: State<'_, UpdateService>, version: String) -> Result<UpdateSnapshot, AppError> {
    let _operation = service.operation.try_lock().map_err(|_| "An update operation is already running.".to_owned())?;
    let cancellation = CancellationToken::new();
    let update = {
        let mut session = service.lock()?;
        session.snapshot.state.require_version(&version, false)?;
        let update = session.pending.clone().ok_or("The selected update is missing.")?;
        session.cancellation = Some(cancellation.clone());
        update
    };
    let release = UpdateMetadata::from(&update);
    service.publish(&app, UpdateState::Downloading { release: release.clone(), downloaded: 0, total: None })?;
    let oversized = AtomicBool::new(false);
    let mut downloaded = 0_u64;
    let mut last_event = Instant::now();
    let download = update.download(
        |length, total| {
            downloaded = downloaded.saturating_add(length as u64);
            if downloaded > MAX_DOWNLOAD_BYTES || total.is_some_and(|total| total > MAX_DOWNLOAD_BYTES) {
                oversized.store(true, Ordering::Relaxed);
                cancellation.cancel();
                return;
            }
            if last_event.elapsed() >= Duration::from_millis(150) {
                let state = UpdateState::Downloading { release: release.clone(), downloaded, total };
                if let Err(error) = service.publish(&app, state) {
                    log::error!("Application update progress failed: {error}");
                    cancellation.cancel();
                }
                last_event = Instant::now();
            }
        },
        || {
            if let Err(error) = service.publish(&app, UpdateState::Verifying { release: release.clone() }) {
                log::error!("Application update verification state failed: {error}");
                cancellation.cancel();
            }
        },
    );
    let result = tokio::select! {
        biased;
        _ = cancellation.cancelled() => Ok(None),
        result = download => result.map(Some),
    };
    service.lock()?.cancellation = None;
    if oversized.load(Ordering::Relaxed) {
        service.fail(&app, UpdateFailure::Download, Error::Network("The update exceeds the 1 GiB download limit.".into()))?;
        return Ok(service.snapshot()?);
    }
    match result {
        Ok(_) if cancellation.is_cancelled() => {
            service.publish(&app, UpdateState::Available { release })?;
        }
        Ok(Some(bytes)) => {
            service.lock()?.bytes = Some(bytes);
            service.publish(&app, UpdateState::Ready { release })?;
        }
        Ok(None) => {
            service.publish(&app, UpdateState::Available { release })?;
        }
        Err(error) => {
            let reason = if matches!(&error, Error::Minisign(_) | Error::Base64(_) | Error::SignatureUtf8(_)) {
                UpdateFailure::Signature
            } else {
                UpdateFailure::Download
            };
            service.fail(&app, reason, error)?;
            return Ok(service.snapshot()?);
        }
    }
    Ok(service.snapshot()?)
}

use tauri::{AppHandle, Manager};

use super::contracts::{UpdateFailure, UpdatePhase};
use super::failure;
use super::state::{busy, require_version, UpdateState};
use crate::app_error::{AppError, AppResult};

pub(super) async fn run(app: &AppHandle, version: &str) -> AppResult<()> {
    let state = app.state::<UpdateState>();
    let _operation = state.operation.try_lock().map_err(|_| busy())?;
    let (update, bytes) = {
        let mut session = state.lock()?;
        require_version(&session.snapshot, UpdatePhase::Ready, version)?;
        let update = session
            .pending
            .clone()
            .ok_or_else(|| AppError::internal("Missing selected update"))?;
        let bytes = session
            .bytes
            .take()
            .ok_or_else(|| AppError::internal("Missing verified update"))?;
        (update, bytes)
    };
    state.publish(|session| {
        session.snapshot.phase = UpdatePhase::Installing;
        session.snapshot.failure = None;
    })?;
    // Only the explicit install action reaches here, after the WebView has committed
    // its close hooks. Windows installers exit the process from inside install().
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let outcome = update.install(&bytes);
        (bytes, outcome)
    })
    .await;
    match outcome {
        Ok((_, Ok(()))) => crate::desktop::restart_after_update(app),
        Ok((bytes, Err(error))) => {
            let (kind, error) = failure::classify(error);
            state.publish(|session| {
                session.bytes = Some(bytes);
                session.snapshot.phase = UpdatePhase::Ready;
                session.snapshot.failure = Some(kind);
            })?;
            Err(error)
        }
        Err(_) => {
            state.publish(|session| {
                session.snapshot.phase = UpdatePhase::Available;
                session.snapshot.failure = Some(UpdateFailure::Internal);
            })?;
            Err(AppError::internal("Update installer task failed"))
        }
    }
}

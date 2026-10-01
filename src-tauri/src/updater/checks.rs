use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::app_error::{AppError, AppResult};
use super::{config, failure};
use super::contracts::{UpdateFailure, UpdateMetadata, UpdatePhase, UpdateSnapshot};
use super::state::{busy, UpdateState};

pub(super) async fn run(app: &AppHandle) -> AppResult<UpdateSnapshot> {
    let state = app.state::<UpdateState>();
    let _operation = state.operation.try_lock().map_err(|_| busy())?;
    if !state.snapshot()?.configured {
        return Err(AppError::not_configured("Updates are disabled for this build"));
    }
    state.publish(|session| {
        session.pending = None;
        session.bytes = None;
        session.snapshot.phase = UpdatePhase::Checking;
        session.snapshot.update = None;
        session.snapshot.failure = None;
        session.snapshot.downloaded = 0;
        session.snapshot.total = None;
    })?;
    let result = config::check(app).await.map_err(failure::classify);
    let update = match result {
        Ok(update) => update,
        Err((kind, error)) => {
            state.publish(|session| {
                session.snapshot.phase = UpdatePhase::Idle;
                session.snapshot.failure = Some(kind);
            })?;
            return Err(error);
        }
    };
    if let Some(update) = &update {
        if config::validate_url(&update.download_url).is_err() || update.version.contains('-') {
            state.publish(|session| {
                session.snapshot.phase = UpdatePhase::Idle;
                session.snapshot.failure = Some(UpdateFailure::Metadata);
            })?;
            return Err(AppError::invalid("Stable update has an invalid download URL or version"));
        }
    }
    state.publish(|session| {
        session.snapshot.update = update.as_ref().map(|update| UpdateMetadata {
            version: update.version.clone(),
            current_version: update.current_version.clone(),
            notes: update.body.clone(),
        });
        session.snapshot.phase = if update.is_some() { UpdatePhase::Available } else { UpdatePhase::Idle };
        session.pending = update;
    })
}

pub(super) fn schedule(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;
        loop {
            let state = app.state::<UpdateState>();
            if matches!(state.snapshot(), Ok(snapshot) if snapshot.phase == UpdatePhase::Idle) {
                if let Err(error) = run(&app).await {
                    log::info!("Automatic update check did not complete: {}", error.code);
                }
            }
            // No catch-up bursts after sleep, no install/download without user action.
            tokio::time::sleep(Duration::from_secs(6 * 60 * 60)).await;
        }
    });
}

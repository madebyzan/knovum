mod checks;
mod config;
mod contracts;
mod download;
mod failure;
mod install;
mod state;

use tauri::{ipc::Channel, AppHandle, Manager};

use crate::app_error::AppResult;
pub use contracts::UpdateSnapshot;
use state::UpdateState;

pub(crate) fn initialize(app: &AppHandle) -> AppResult<()> {
    let configured = config::initialize(app)?;
    app.manage(UpdateState::new(configured));
    if configured {
        checks::schedule(app.clone());
    }
    Ok(())
}

#[tauri::command]
pub fn observe_updates(app: AppHandle, on_event: Channel<UpdateSnapshot>) -> AppResult<UpdateSnapshot> {
    let state = app.state::<UpdateState>();
    let mut session = state.lock()?;
    session.observer = Some(on_event);
    Ok(session.snapshot.clone())
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> AppResult<UpdateSnapshot> {
    checks::run(&app).await
}

#[tauri::command]
pub async fn download_update(app: AppHandle, version: String) -> AppResult<UpdateSnapshot> {
    download::run(&app, &version).await
}

#[tauri::command]
pub fn cancel_update_download(app: AppHandle) -> AppResult<()> {
    let state = app.state::<UpdateState>();
    if let Some(token) = &state.lock()?.cancellation {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub async fn install_update(app: AppHandle, version: String) -> AppResult<()> {
    install::run(&app, &version).await
}

#[cfg(test)]
mod tests;

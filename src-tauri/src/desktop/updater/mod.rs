mod contracts;
pub(crate) mod download;
mod state;

use std::time::Duration;

use tauri::{AppHandle, Manager, State};
use tauri_plugin_updater::{Error, UpdaterExt};

use crate::foundation::app_error::AppError;
pub use contracts::{UpdateFailure, UpdateMetadata, UpdateSnapshot, UpdateState};
pub use state::UpdateService;

pub(crate) fn setup(app: &AppHandle) -> tauri::Result<()> {
    let configured = app.config().plugins.0.contains_key("updater");
    if configured {
        app.plugin(tauri_plugin_updater::Builder::new().build())?;
    }
    app.manage(UpdateService::new(configured));
    if configured {
        start_checks(app.clone());
    }
    Ok(())
}

fn start_checks(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let mut interval = tokio::time::interval(Duration::from_secs(6 * 60 * 60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let service = app.state::<UpdateService>();
            if service
                .snapshot()
                .is_ok_and(|snapshot| snapshot.state.can_check())
            {
                if let Err(error) = check(&app, &service).await {
                    log::debug!("Scheduled application update check did not complete: {error}");
                }
            }
        }
    });
}

#[tauri::command]
pub fn get_update_state(service: State<'_, UpdateService>) -> Result<UpdateSnapshot, AppError> {
    Ok(service.snapshot()?)
}

#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    service: State<'_, UpdateService>,
) -> Result<UpdateSnapshot, AppError> {
    check(&app, &service).await
}

async fn check(app: &AppHandle, service: &UpdateService) -> Result<UpdateSnapshot, AppError> {
    let _operation = service
        .operation
        .try_lock()
        .map_err(|_| "An update operation is already running.".to_owned())?;
    if !service.snapshot()?.state.can_check() {
        return Err("Finish the current update before checking again.".into());
    }
    service.publish(app, UpdateState::Checking)?;
    let installation = async {
        let updater = app
            .updater_builder()
            .timeout(Duration::from_secs(30))
            .configure_client(|client| {
                client
                    .connect_timeout(Duration::from_secs(15))
                    .read_timeout(Duration::from_secs(60))
                    .https_only(true)
            })
            .build()?;
        let mut update = updater.check().await?;
        if let Some(update) = &mut update {
            if update.download_url.scheme() != "https"
                || !update.download_url.username().is_empty()
                || update.download_url.password().is_some()
            {
                return Err(Error::InsecureTransportProtocol);
            }
            update.timeout = Some(Duration::from_secs(30 * 60));
        }
        Ok::<_, Error>(update)
    }
    .await;
    match installation {
        Ok(update) => {
            let state = match &update {
                Some(update) => UpdateState::Available {
                    release: update.into(),
                },
                None => UpdateState::Current,
            };
            service.lock()?.pending = update;
            service.publish(app, state)?;
            Ok(service.snapshot()?)
        }
        Err(error) => {
            service.fail(app, UpdateFailure::Check, error)?;
            Ok(service.snapshot()?)
        }
    }
}

#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    service: State<'_, UpdateService>,
    version: String,
) -> Result<UpdateSnapshot, AppError> {
    let _operation = service
        .operation
        .try_lock()
        .map_err(|_| "An update operation is already running.".to_owned())?;
    let (update, bytes, release) = {
        let mut session = service.lock()?;
        session.snapshot.state.require_version(&version, true)?;
        let update = session
            .pending
            .take()
            .ok_or("The verified update is missing.")?;
        let bytes = session
            .bytes
            .take()
            .ok_or("The verified download is missing.")?;
        let release = UpdateMetadata::from(&update);
        (update, bytes, release)
    };
    service.publish(&app, UpdateState::Installing { release })?;
    let installation = tauri::async_runtime::spawn_blocking(move || update.install(bytes)).await;
    match installation {
        Ok(Ok(())) => app.restart(),
        Ok(Err(error)) => {
            service.fail(&app, UpdateFailure::Install, error)?;
            Ok(service.snapshot()?)
        }
        Err(error) => {
            log::error!("Application update installer task failed: {error}");
            service.publish(
                &app,
                UpdateState::Failed {
                    reason: UpdateFailure::Install,
                },
            )?;
            Ok(service.snapshot()?)
        }
    }
}

#[cfg(test)]
mod tests;

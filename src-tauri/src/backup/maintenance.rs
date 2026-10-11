use std::path::Path;
use tauri::{AppHandle, Runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use super::{
    archive,
    contracts::{MaintenanceOperation, MaintenanceReceipt, MaintenanceStatus, PendingOperation},
    paths::{self, AppPaths},
    publish, staging,
};

pub fn is_required<R: Runtime>(app: &AppHandle<R>) -> bool {
    match AppPaths::resolve(app) {
        Ok(paths) => {
            paths::entry_exists(&paths.pending()).unwrap_or(true)
                || publish::has_unreconciled(&paths)
                || staging::has_owned(&paths)
        }
        Err(_) => true,
    }
}

pub async fn run<R: Runtime>(app: &AppHandle<R>) -> Result<bool, String> {
    let app_paths = AppPaths::resolve(app)?;
    let recovery = match publish::reconcile(&app_paths) {
        Ok(recovery) => recovery,
        Err(error) => {
            let receipt = MaintenanceReceipt::failure(
                MaintenanceOperation::Restore,
                format!("Restore recovery failed: {error}"),
            );
            if !paths::entry_exists(&app_paths.receipt())? {
                paths::write_receipt(&app_paths, &receipt)?;
            }
            return Err(receipt.message);
        }
    };
    if let super::journal::ReconcileOutcome::RecoveredCommit { cleanup_warning } = recovery {
        let receipt = MaintenanceReceipt::success(
            MaintenanceOperation::Restore,
            success_message(MaintenanceOperation::Restore),
            cleanup_warning,
        );
        paths::write_receipt(&app_paths, &receipt)?;
        paths::clear_pending(&app_paths)?;
        return Ok(true);
    }
    let operation = paths::read_pending(&app_paths)?;
    if let Err(error) = staging::cleanup_abandoned(&app_paths) {
        if let Some(operation) = &operation {
            let operation_name = match operation {
                PendingOperation::Backup { .. } => MaintenanceOperation::Backup,
                PendingOperation::Restore { .. } => MaintenanceOperation::Restore,
            };
            let receipt = MaintenanceReceipt::failure(
                operation_name,
                format!("Data maintenance staging recovery failed: {error}"),
            );
            paths::write_receipt(&app_paths, &receipt)?;
            return Err(receipt.message);
        }
        return Err(format!("Data maintenance staging recovery failed: {error}"));
    }
    let Some(operation) = operation else {
        return Ok(true);
    };
    let operation_name = match &operation {
        PendingOperation::Backup { .. } => MaintenanceOperation::Backup,
        PendingOperation::Restore { .. } => MaintenanceOperation::Restore,
    };
    set_progress(app, 20);
    let outcome = perform(&app_paths, operation).await;
    set_progress(app, 90);
    let succeeded = outcome.is_ok();
    let receipt = match outcome {
        Ok(cleanup_warning) => MaintenanceReceipt::success(
            operation_name,
            success_message(operation_name),
            cleanup_warning,
        ),
        Err(error) => MaintenanceReceipt::failure(operation_name, error),
    };
    paths::write_receipt(&app_paths, &receipt)?;
    if receipt.status == MaintenanceStatus::Failed {
        if publish::has_unreconciled(&app_paths) {
            return Err(receipt.message);
        }
        show_error(app, &receipt.message);
    }
    paths::clear_pending(&app_paths)?;
    Ok(succeeded)
}

async fn perform(paths: &AppPaths, operation: PendingOperation) -> Result<Option<String>, String> {
    match operation {
        PendingOperation::Backup { destination } => {
            archive::create(paths, Path::new(&destination)).await
        }
        PendingOperation::Restore { source } => {
            let (manifest, config_stage, data_stage) = archive::extract(paths, Path::new(&source))?;
            let outcome = publish::publish(paths, &manifest, &config_stage, &data_stage).await;
            match outcome {
                Ok(cleanup_warning) => Ok(cleanup_warning),
                Err(error) => {
                    if let Err(cleanup_error) = staging::cleanup_restore(&config_stage, &data_stage)
                    {
                        return Err(format!(
                            "{error}; restore staging cleanup failed: {cleanup_error}"
                        ));
                    }
                    Err(error)
                }
            }
        }
    }
}

pub(super) fn show_error<R: Runtime>(app: &AppHandle<R>, message: &str) {
    app.dialog()
        .message(message)
        .title("Knovum data maintenance failed")
        .kind(MessageDialogKind::Error)
        .show(|_| {});
}

fn set_progress<R: Runtime>(app: &AppHandle<R>, progress: u8) {
    if let Err(error) = crate::desktop::set_maintenance_progress(app, Some(progress)) {
        log::warn!("failed to update maintenance progress: {error}");
    }
}

fn success_message(operation: MaintenanceOperation) -> &'static str {
    if matches!(operation, MaintenanceOperation::Restore) {
        "Backup restored. Credentials were excluded and must be configured again."
    } else {
        "Backup created successfully. Credentials and downloaded runtime files were excluded."
    }
}

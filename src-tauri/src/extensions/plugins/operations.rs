use crate::{
    app_error::{AppError, AppResult},
    job_registry::{JobLease, JobRegistry},
};
use std::{
    future::Future,
    sync::{Arc, Mutex, OnceLock},
};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::super::contracts::PluginOperationStage;

struct Control {
    cancellation: CancellationToken,
    committing: Mutex<bool>,
}

fn registry() -> &'static JobRegistry<Arc<Control>> {
    static REGISTRY: OnceLock<JobRegistry<Arc<Control>>> = OnceLock::new();
    REGISTRY.get_or_init(|| JobRegistry::new("Plugin operation"))
}

pub(super) struct PluginOperation {
    control: Arc<Control>,
    progress: Channel<PluginOperationStage>,
    _lease: JobLease<Arc<Control>>,
}

impl PluginOperation {
    pub(super) fn begin(
        id: &str,
        progress: Channel<PluginOperationStage>,
        parent: &CancellationToken,
    ) -> AppResult<Self> {
        if id.trim().is_empty() {
            return Err(AppError::invalid(
                "Plugin operation requires a job identifier",
            ));
        }
        let control = Arc::new(Control {
            cancellation: parent.child_token(),
            committing: Mutex::new(false),
        });
        let lease = registry().acquire(id, control.clone())?;
        Ok(Self {
            control,
            progress,
            _lease: lease,
        })
    }

    pub(super) fn cancellation(&self) -> &CancellationToken {
        &self.control.cancellation
    }

    pub(super) async fn read<T>(&self, future: impl Future<Output = AppResult<T>>) -> AppResult<T> {
        self.observe(PluginOperationStage::Reading);
        let operation = tokio::select! {
            biased;
            _ = self.control.cancellation.cancelled() => {
                Err(AppError::cancelled("Plugin operation was cancelled"))
            },
            operation = future => operation,
        }?;
        self.observe(PluginOperationStage::Validating);
        Ok(operation)
    }

    pub(super) fn commit(&self) -> AppResult<()> {
        let mut committing = self
            .control
            .committing
            .lock()
            .map_err(|error| error.to_string())?;
        if self.control.cancellation.is_cancelled() {
            return Err(AppError::cancelled("Plugin operation was cancelled"));
        }
        *committing = true;
        self.observe(PluginOperationStage::Committing);
        Ok(())
    }

    fn observe(&self, stage: PluginOperationStage) {
        if let Err(error) = self.progress.send(stage) {
            log::debug!("plugin operation observer disconnected: {error}");
        }
    }
}

impl Drop for PluginOperation {
    fn drop(&mut self) {
        self.control.cancellation.cancel();
    }
}

#[tauri::command]
pub fn cancel_plugin_operation(job_id: String) -> AppResult<()> {
    let control = registry().get(&job_id)?;
    let committing = control
        .committing
        .lock()
        .map_err(|error| error.to_string())?;
    if *committing {
        return Err(AppError::new(
            "plugin_operation_committing",
            "Plugin operation is already committing",
        ));
    }
    control.cancellation.cancel();
    Ok(())
}

#[cfg(test)]
#[path = "tests/operations.rs"]
mod tests;

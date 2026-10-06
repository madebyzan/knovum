use crate::job_registry::{JobLease, JobRegistry};
use std::{
    future::Future,
    sync::{Arc, Mutex, OnceLock},
};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::super::contracts::PluginOperationStage;

pub(crate) const CANCELLED: &str = "plugin-operation-cancelled";

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
    ) -> Result<Self, String> {
        if id.trim().is_empty() {
            return Err("plugin-operation-id-required".into());
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

    pub(super) async fn read<T>(
        &self,
        future: impl Future<Output = Result<T, String>>,
    ) -> Result<T, String> {
        self.observe(PluginOperationStage::Reading);
        let result = tokio::select! {
            biased;
            _ = self.control.cancellation.cancelled() => Err(CANCELLED.into()),
            result = future => result,
        }?;
        self.observe(PluginOperationStage::Validating);
        Ok(result)
    }

    pub(super) fn commit(&self) -> Result<(), String> {
        let mut committing = self
            .control
            .committing
            .lock()
            .map_err(|error| error.to_string())?;
        if self.control.cancellation.is_cancelled() {
            return Err(CANCELLED.into());
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
pub fn cancel_plugin_operation(job_id: String) -> Result<(), String> {
    let control = registry().get(&job_id)?;
    let committing = control
        .committing
        .lock()
        .map_err(|error| error.to_string())?;
    if *committing {
        return Err("plugin-operation-already-committing".into());
    }
    control.cancellation.cancel();
    Ok(())
}

#[cfg(test)]
#[path = "tests/operations.rs"]
mod tests;

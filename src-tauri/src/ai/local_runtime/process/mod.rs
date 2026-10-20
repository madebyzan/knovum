mod diagnostics;
mod environment;
mod package;
pub(super) use environment::isolate_environment;
pub(in crate::ai::local_runtime) use package::WorkerPackage;
mod request;
mod startup;
use crate::foundation::subprocess::process_tree::{KillHandle, ProcessTree};
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use tokio::process::Child;

use super::protocol::Protocol;
use crate::ai::contracts::runtime::{
    RuntimeCapability, RuntimeHealthObservation, RuntimeHealthParams,
};

#[cfg(windows)]
use crate::foundation::subprocess::windows_job;

/// What a capability decides about the worker it needs.
///
/// Everything else about starting a worker — its session identity, its private work directory and
/// the budget of its cold start — is derived by [`Worker::start`], so no capability can describe
/// those differently from another.
pub(in crate::ai::local_runtime) struct WorkerSpecification<'a> {
    pub(in crate::ai::local_runtime) capability: RuntimeCapability,
    pub(in crate::ai::local_runtime) capability_spec_id: String,
    pub(in crate::ai::local_runtime) device_profile:
        crate::ai::contracts::runtime::RuntimeDeviceProfile,
    /// The verified snapshot this worker loads. Worker reuse and its initialization budget are
    /// both measured against it.
    pub(in crate::ai::local_runtime) model_path: &'a Path,
    pub(in crate::ai::local_runtime) model_paths: crate::ai::contracts::runtime::RuntimeModelPaths,
}

pub(super) struct Worker {
    pub(super) child: Child,
    process_tree: ProcessTree,
    memory_reservation: Option<super::admission::MemoryReservation>,
    _kill_registration: KillRegistration,
    pub(super) protocol: Protocol,
    pub(super) session_id: String,
    pub(super) runtime_digest: String,
    pub(super) model_path: PathBuf,
    pub(super) capability: RuntimeCapability,
    pub(super) capability_spec_id: String,
    pub(super) device_profile: crate::ai::contracts::runtime::RuntimeDeviceProfile,
    work_directory: PathBuf,
    pub(super) last_used: Instant,
}

impl Worker {
    pub(super) fn work_directory(&self) -> &Path {
        &self.work_directory
    }

    pub(super) fn matches(
        &self,
        runtime_digest: &str,
        model_path: &Path,
        capability: RuntimeCapability,
        capability_spec_id: &str,
        device_profile: &crate::ai::contracts::runtime::RuntimeDeviceProfile,
    ) -> bool {
        self.runtime_digest == runtime_digest
            && self.model_path == model_path
            && self.capability == capability
            && self.capability_spec_id == capability_spec_id
            && self.device_profile == *device_profile
    }

    pub(super) async fn resident_bytes(&mut self) -> Result<u64, String> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let health = tokio::time::timeout(
            Duration::from_secs(2),
            self.protocol.request::<_, RuntimeHealthObservation>(
                &request_id,
                "runtime.health",
                &RuntimeHealthParams {
                    session_id: self.session_id.clone(),
                },
            ),
        )
        .await
        .map_err(|_| "deadline_exceeded: runtime health timed out".to_owned())??;
        if health.session_id != self.session_id || health.current_request_id.is_some() {
            return Err("invalid_output: runtime health identity or worker state differs".into());
        }
        if health.resident_bytes == 0 {
            return Err("invalid_output: runtime health reported zero resident memory".into());
        }
        Ok(health.resident_bytes)
    }

    pub(super) async fn terminate(&mut self) {
        self.process_tree.terminate();
        self._kill_registration.clear();
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
        let _ = tokio::fs::remove_dir_all(&self.work_directory).await;
        if let Some(mut reservation) = self.memory_reservation.take() {
            reservation.release();
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.process_tree.terminate();
        self._kill_registration.clear();
    }
}

static ACTIVE_KILL: OnceLock<Mutex<Option<(String, KillHandle)>>> = OnceLock::new();

struct KillRegistration {
    session_id: String,
}

impl KillRegistration {
    fn new(session_id: &str, handle: KillHandle) -> Self {
        let registry = ACTIVE_KILL.get_or_init(|| Mutex::new(None));
        if let Ok(mut active) = registry.lock() {
            *active = Some((session_id.to_owned(), handle));
        }
        Self {
            session_id: session_id.to_owned(),
        }
    }

    fn clear(&self) {
        if let Some(registry) = ACTIVE_KILL.get() {
            if let Ok(mut active) = registry.lock() {
                if active
                    .as_ref()
                    .is_some_and(|(id, _)| id == &self.session_id)
                {
                    *active = None;
                }
            }
        }
    }
}

impl Drop for KillRegistration {
    fn drop(&mut self) {
        self.clear();
    }
}

pub(super) fn force_terminate_active() {
    if let Some(registry) = ACTIVE_KILL.get() {
        if let Ok(active) = registry.lock() {
            if let Some((_, handle)) = active.as_ref() {
                handle.terminate();
            }
        }
    }
}

#[cfg(test)]
mod tests;

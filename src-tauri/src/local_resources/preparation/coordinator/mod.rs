use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
};

use tauri::{AppHandle, Runtime};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use super::contracts::{ResourcePreparationProgress, ResourcePreparationStage};
use super::{installation, RESOURCE_PREPARATION_CANCELLED};
use crate::local_resources::models::{
    catalog::{self, ModelKind},
    snapshots::SnapshotKind,
};

#[derive(Clone)]
pub(super) enum InstallationState {
    Progress(ResourcePreparationProgress),
    Failed(Arc<str>),
}

pub(super) struct Installation {
    pub(super) cancellation: CancellationToken,
    state: watch::Sender<InstallationState>,
    waiters: AtomicUsize,
    terminal: AtomicBool,
}

impl Installation {
    fn new(kind: ModelKind) -> Arc<Self> {
        let (state, _) = watch::channel(InstallationState::Progress(ResourcePreparationProgress {
            stage: ResourcePreparationStage::Queued,
            downloaded: 0,
            total: Some(catalog::descriptor(kind).snapshot.total_bytes()),
            percent: Some(0),
        }));
        Arc::new(Self {
            cancellation: CancellationToken::new(),
            state,
            waiters: AtomicUsize::new(0),
            terminal: AtomicBool::new(false),
        })
    }

    fn join(self: &Arc<Self>) -> InstallationWaiter {
        self.waiters.fetch_add(1, Ordering::AcqRel);
        InstallationWaiter {
            installation: Arc::clone(self),
            state: self.state.subscribe(),
        }
    }

    pub(super) fn publish(&self, state: InstallationState) {
        self.state.send_replace(state);
    }

    fn finish(&self, state: InstallationState) {
        self.terminal.store(true, Ordering::Release);
        self.publish(state);
    }
}

pub(super) struct InstallationWaiter {
    installation: Arc<Installation>,
    state: watch::Receiver<InstallationState>,
}

impl InstallationWaiter {
    pub(super) async fn wait(
        &mut self,
        request_cancellation: &CancellationToken,
        progress: &mut impl FnMut(ResourcePreparationProgress),
    ) -> Result<(), String> {
        loop {
            if request_cancellation.is_cancelled() {
                return Err(RESOURCE_PREPARATION_CANCELLED.to_string());
            }
            match self.state.borrow_and_update().clone() {
                InstallationState::Progress(update) => {
                    progress(update.clone());
                    if matches!(update.stage, ResourcePreparationStage::Ready) {
                        return Ok(());
                    }
                }
                InstallationState::Failed(error) => return Err(error.to_string()),
            }
            tokio::select! {
                biased;
                _ = request_cancellation.cancelled() => {
                    return Err(RESOURCE_PREPARATION_CANCELLED.to_string());
                }
                changed = self.state.changed() => {
                    if changed.is_err() {
                        return Err("Model preparation ended without a terminal state".to_string());
                    }
                }
            }
        }
    }
}

impl Drop for InstallationWaiter {
    fn drop(&mut self) {
        let Ok(_active) = installations().lock() else {
            self.installation.cancellation.cancel();
            return;
        };
        if self.installation.waiters.fetch_sub(1, Ordering::AcqRel) == 1
            && !self.installation.terminal.load(Ordering::Acquire)
        {
            self.installation.cancellation.cancel();
        }
    }
}

pub(super) async fn join_installation<R: Runtime>(
    app: &AppHandle<R>,
    kind: ModelKind,
    request_cancellation: &CancellationToken,
) -> Result<InstallationWaiter, String> {
    let snapshot = catalog::descriptor(kind).snapshot;
    loop {
        let outcome = {
            let mut active = installations()
                .lock()
                .map_err(|_| "Model preparation lock poisoned".to_string())?;
            match active.get(&snapshot) {
                Some(existing) if existing.cancellation.is_cancelled() => {
                    Err(existing.state.subscribe())
                }
                Some(existing) => Ok((Arc::clone(existing), existing.join(), false)),
                None => {
                    let installation = Installation::new(kind);
                    let waiter = installation.join();
                    active.insert(snapshot, Arc::clone(&installation));
                    Ok((installation, waiter, true))
                }
            }
        };
        match outcome {
            Ok((installation, waiter, should_start)) => {
                if should_start {
                    installation::start(app.clone(), snapshot, installation);
                }
                return Ok(waiter);
            }
            Err(mut retirement) => {
                tokio::select! {
                    biased;
                    _ = request_cancellation.cancelled() => {
                        return Err(RESOURCE_PREPARATION_CANCELLED.to_string());
                    }
                    changed = retirement.changed() => {
                        if changed.is_err() {
                            return Err("Model installation retired without a terminal state".to_string());
                        }
                    }
                }
            }
        }
    }
}

pub(super) fn complete(
    snapshot: SnapshotKind,
    installation: &Arc<Installation>,
    state: InstallationState,
) {
    if let Ok(mut active) = installations().lock() {
        installation.finish(state);
        if active
            .get(&snapshot)
            .is_some_and(|current| Arc::ptr_eq(current, installation))
        {
            active.remove(&snapshot);
        }
    } else {
        installation.finish(InstallationState::Failed(
            "Model preparation lock poisoned".into(),
        ));
    }
}

fn installations() -> &'static Mutex<HashMap<SnapshotKind, Arc<Installation>>> {
    static INSTALLATIONS: OnceLock<Mutex<HashMap<SnapshotKind, Arc<Installation>>>> =
        OnceLock::new();
    INSTALLATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
mod tests;

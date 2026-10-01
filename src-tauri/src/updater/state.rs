use std::sync::{Mutex, MutexGuard};

use tauri::ipc::Channel;
use tauri_plugin_updater::Update;
use tokio_util::sync::CancellationToken;

use crate::app_error::{AppError, AppResult};
use super::contracts::{UpdateFailure, UpdatePhase, UpdateSnapshot};

pub(super) enum DownloadOutcome {
    Verified(Vec<u8>),
    Failed(UpdateFailure),
    Cancelled,
}

pub(super) struct Session {
    pub snapshot: UpdateSnapshot,
    pub pending: Option<Update>,
    pub bytes: Option<Vec<u8>>,
    pub cancellation: Option<CancellationToken>,
    pub observer: Option<Channel<UpdateSnapshot>>,
}

pub(super) struct UpdateState {
    session: Mutex<Session>,
    pub operation: tokio::sync::Mutex<()>,
}

impl UpdateState {
    pub fn new(configured: bool) -> Self {
        Self {
            session: Mutex::new(Session {
                snapshot: UpdateSnapshot::new(configured),
                pending: None,
                bytes: None,
                cancellation: None,
                observer: None,
            }),
            operation: tokio::sync::Mutex::new(()),
        }
    }

    pub fn lock(&self) -> AppResult<MutexGuard<'_, Session>> {
        self.session.lock().map_err(|_| AppError::internal("Updater state lock poisoned"))
    }

    pub fn snapshot(&self) -> AppResult<UpdateSnapshot> {
        Ok(self.lock()?.snapshot.clone())
    }

    pub fn publish(&self, change: impl FnOnce(&mut Session)) -> AppResult<UpdateSnapshot> {
        let mut session = self.lock()?;
        change(&mut session);
        session.snapshot.revision += 1;
        let snapshot = session.snapshot.clone();
        if let Some(observer) = &session.observer {
            if observer.send(snapshot.clone()).is_err() {
                // A detached WebView does not cancel a native operation. A new observer
                // receives the current snapshot atomically from observe_updates.
                session.observer = None;
                log::info!("Updater observer detached");
            }
        }
        Ok(snapshot)
    }

    pub fn complete_download(&self, outcome: DownloadOutcome) -> AppResult<UpdateSnapshot> {
        self.publish(|session| {
            // Cancellation and Ready publication share this lock. Once cancellation
            // is accepted, even a just-verified download cannot become installable.
            let outcome = if session.cancellation.as_ref().is_some_and(|token| token.is_cancelled()) {
                DownloadOutcome::Cancelled
            } else {
                outcome
            };
            session.cancellation = None;
            session.bytes = None;
            session.snapshot.phase = UpdatePhase::Available;
            session.snapshot.failure = None;
            session.snapshot.downloaded = 0;
            session.snapshot.total = None;
            match outcome {
                DownloadOutcome::Verified(bytes) => {
                    session.snapshot.phase = UpdatePhase::Ready;
                    session.snapshot.downloaded = bytes.len() as u64;
                    session.snapshot.total = Some(bytes.len() as u64);
                    session.bytes = Some(bytes);
                }
                DownloadOutcome::Failed(failure) => session.snapshot.failure = Some(failure),
                DownloadOutcome::Cancelled => {}
            }
        })
    }

}

pub(super) fn require_version(snapshot: &UpdateSnapshot, phase: UpdatePhase, version: &str) -> AppResult<()> {
    if snapshot.phase != phase {
        return Err(AppError::invalid("Update is not in the required phase"));
    }
    if snapshot.update.as_ref().map(|value| value.version.as_str()) != Some(version) {
        return Err(AppError::invalid("The selected update version has changed; check again"));
    }
    Ok(())
}

pub(super) fn busy() -> AppError {
    AppError::new("update_busy", "Another update operation is in progress")
}

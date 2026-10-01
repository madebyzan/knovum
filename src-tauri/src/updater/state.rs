use std::sync::{Mutex, MutexGuard};

use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::Update;
use tokio_util::sync::CancellationToken;

use super::contracts::{UpdateFailure, UpdateSnapshot, UpdateState};

pub(super) struct Session {
    pub snapshot: UpdateSnapshot,
    pub pending: Option<Update>,
    pub bytes: Option<Vec<u8>>,
    pub cancellation: Option<CancellationToken>,
}

pub struct UpdateService {
    pub(super) operation: tokio::sync::Mutex<()>,
    session: Mutex<Session>,
}

impl UpdateService {
    pub(super) fn new(configured: bool) -> Self {
        Self {
            operation: tokio::sync::Mutex::new(()),
            session: Mutex::new(Session {
                snapshot: UpdateSnapshot {
                    revision: 0,
                    state: if configured { UpdateState::Idle } else { UpdateState::Disabled },
                },
                pending: None,
                bytes: None,
                cancellation: None,
            }),
        }
    }

    pub(crate) fn blocks_exit(&self) -> bool {
        match self.snapshot() {
            Ok(snapshot) => matches!(snapshot.state, UpdateState::Installing { .. }),
            Err(error) => {
                log::error!("Cannot determine whether the updater is replacing application files: {error}");
                true
            }
        }
    }

    pub(super) fn lock(&self) -> Result<MutexGuard<'_, Session>, String> {
        self.session.lock().map_err(|_| "Application updater state is poisoned.".to_owned())
    }

    pub(super) fn snapshot(&self) -> Result<UpdateSnapshot, String> {
        Ok(self.lock()?.snapshot.clone())
    }

    pub(super) fn publish(&self, app: &AppHandle, state: UpdateState) -> Result<(), String> {
        let snapshot = {
            let mut session = self.lock()?;
            session.snapshot.revision += 1;
            session.snapshot.state = state;
            session.snapshot.clone()
        };
        if let Err(error) = app.emit_to("main", "knovum://update-state", &snapshot) {
            log::warn!("Failed to deliver updater state: {error}");
        }
        Ok(())
    }

    pub(super) fn fail(
        &self,
        app: &AppHandle,
        reason: UpdateFailure,
        error: tauri_plugin_updater::Error,
    ) -> Result<(), String> {
        let diagnostic = match error {
            tauri_plugin_updater::Error::Reqwest(error) => error.without_url().to_string(),
            tauri_plugin_updater::Error::SignatureUtf8(_) => "Invalid signature encoding".into(),
            error => error.to_string(),
        };
        log::error!("Application update {reason:?} failed: {diagnostic}");
        {
            let mut session = self.lock()?;
            session.pending = None;
            session.bytes = None;
            session.cancellation = None;
        }
        self.publish(app, UpdateState::Failed { reason })
    }
}

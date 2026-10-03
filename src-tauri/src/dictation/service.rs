use std::sync::{Arc, RwLock};

use tauri::{AppHandle, Runtime};
use tokio::sync::mpsc::{self, UnboundedSender};

use super::{
    actor, contracts::DictationSnapshot, insertion::TargetApplication,
    transcription::TranscriptionOutcome,
};

pub(crate) enum DictationAction {
    Press,
    Release,
    Cancel,
    Dismiss,
    Shutdown,
    DeviceFailed {
        session_id: String,
        code: String,
        message: String,
    },
    Finished {
        session_id: String,
        target: TargetApplication,
        result: Result<TranscriptionOutcome, String>,
    },
}

pub(crate) struct DictationService {
    actions: UnboundedSender<DictationAction>,
    snapshot: Arc<RwLock<DictationSnapshot>>,
}

impl DictationService {
    pub(crate) fn start<R: Runtime>(app: AppHandle<R>) -> Self {
        super::lifecycle::cleanup_cache(&app);
        let (actions, receiver) = mpsc::unbounded_channel();
        let snapshot = Arc::new(RwLock::new(DictationSnapshot::default()));
        tauri::async_runtime::spawn(actor::run(
            app,
            receiver,
            actions.clone(),
            Arc::clone(&snapshot),
        ));
        Self { actions, snapshot }
    }

    pub(crate) fn dispatch(&self, action: DictationAction) {
        if self.actions.send(action).is_err() {
            log::warn!("dictation service is no longer available");
        }
    }

    pub(crate) fn snapshot(&self) -> Result<DictationSnapshot, String> {
        self.snapshot
            .read()
            .map(|value| value.clone())
            .map_err(|_| "Dictation state is unavailable".into())
    }
}

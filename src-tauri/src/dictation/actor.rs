use std::sync::{Arc, RwLock};

use tauri::{AppHandle, Runtime};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

use super::{
    completion, contracts::DictationSnapshot, insertion::TargetApplication, lifecycle,
    recorder::Recorder, service::DictationAction,
};

pub(super) enum ActiveSession {
    Idle,
    Recording {
        id: String,
        target: TargetApplication,
        recorder: Recorder,
        wav: std::path::PathBuf,
    },
    Transcribing {
        id: String,
        cancellation: CancellationToken,
    },
}

pub(super) async fn run<R: Runtime>(
    app: AppHandle<R>,
    mut receiver: UnboundedReceiver<DictationAction>,
    sender: UnboundedSender<DictationAction>,
    snapshot: Arc<RwLock<DictationSnapshot>>,
) {
    let mut active = ActiveSession::Idle;
    while let Some(action) = receiver.recv().await {
        match action {
            DictationAction::Press => lifecycle::start(&app, &sender, &snapshot, &mut active),
            DictationAction::Release => {
                lifecycle::stop(&app, &sender, &snapshot, &mut active).await
            }
            DictationAction::Cancel => lifecycle::cancel(&app, &snapshot, &mut active),
            DictationAction::Dismiss => {
                if matches!(active, ActiveSession::Idle) {
                    completion::publish(&app, &snapshot, DictationSnapshot::default());
                }
            }
            DictationAction::DeviceFailed {
                session_id,
                code,
                message,
            } => lifecycle::device_failed(&app, &snapshot, &mut active, session_id, code, message),
            DictationAction::Finished {
                session_id,
                target,
                result,
            } => completion::finish(&app, &snapshot, &mut active, session_id, target, result).await,
            DictationAction::Shutdown => {
                lifecycle::cancel(&app, &snapshot, &mut active);
                break;
            }
        }
    }
}

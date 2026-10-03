use std::sync::{Arc, RwLock};

use tauri::{AppHandle, Emitter, Runtime};

use super::{
    actor::ActiveSession,
    contracts::{DictationPhase, DictationSnapshot, InsertionOutcome},
    insertion::{self, TargetApplication},
    transcription::TranscriptionOutcome,
};

const STATE_EVENT: &str = "knovum://dictation-state";

pub(super) async fn finish<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: &Arc<RwLock<DictationSnapshot>>,
    active: &mut ActiveSession,
    session_id: String,
    target: TargetApplication,
    result: Result<TranscriptionOutcome, String>,
) {
    if !matches!(active, ActiveSession::Transcribing { id, .. } if id == &session_id) {
        return;
    }
    *active = ActiveSession::Idle;
    match result {
        Ok(result) => publish_result(app, snapshot, session_id, target, result).await,
        Err(error) if error == "cancelled" => publish(app, snapshot, DictationSnapshot::default()),
        Err(error) => fail(app, snapshot, Some(session_id), error),
    }
}

async fn publish_result<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: &Arc<RwLock<DictationSnapshot>>,
    session_id: String,
    target: TargetApplication,
    result: TranscriptionOutcome,
) {
    let text = result.text.clone();
    let insertion =
        crate::blocking::run(move || Ok::<_, String>(insertion::insert(&target, &text)))
            .await
            .unwrap_or_else(|error| insertion::InsertionDecision {
                outcome: InsertionOutcome::Failed,
                message: Some(error),
            });
    let needs_attention = !matches!(insertion.outcome, InsertionOutcome::Inserted);
    let error_code = match insertion.outcome {
        InsertionOutcome::Inserted => None,
        InsertionOutcome::NotAttempted => Some("dictation_empty_result"),
        InsertionOutcome::FocusChanged => Some("dictation_focus_changed"),
        InsertionOutcome::PermissionDenied => Some("dictation_permission_denied"),
        InsertionOutcome::Failed => Some("dictation_insertion_failed"),
    };
    publish(
        app,
        snapshot,
        DictationSnapshot {
            phase: DictationPhase::Result,
            session_id: Some(session_id),
            text: Some(result.text),
            language: Some(result.language),
            insertion: Some(insertion.outcome),
            error_code: error_code.map(str::to_owned),
            error_message: insertion.message,
            ..DictationSnapshot::default()
        },
    );
    if needs_attention {
        reveal(app);
    }
}

pub(super) fn fail<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: &Arc<RwLock<DictationSnapshot>>,
    session_id: Option<String>,
    error: String,
) {
    let (code, message) = error
        .split_once(": ")
        .map_or(("dictation_failed", error.as_str()), |(code, message)| {
            (code, message)
        });
    publish(
        app,
        snapshot,
        DictationSnapshot::failed(session_id, code, message.into()),
    );
    reveal(app);
}

pub(super) fn publish<R: Runtime>(
    app: &AppHandle<R>,
    shared: &Arc<RwLock<DictationSnapshot>>,
    value: DictationSnapshot,
) {
    if let Ok(mut snapshot) = shared.write() {
        *snapshot = value.clone();
    }
    if let Err(error) = app.emit(STATE_EVENT, value) {
        log::warn!("failed to emit dictation state: {error}");
    }
}

pub(super) fn reveal<R: Runtime>(app: &AppHandle<R>) {
    if let Err(error) = crate::app_window::focus_main(app) {
        log::warn!("failed to reveal dictation result: {error}");
    }
}

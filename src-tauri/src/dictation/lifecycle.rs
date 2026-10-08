use std::sync::{Arc, RwLock};

use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use super::{
    actor::ActiveSession,
    completion, insertion,
    recorder::{RecordedAudio, Recorder},
    service::DictationAction,
    transcription,
};
use crate::dictation_contracts::{DictationFailure, DictationPhase, DictationSnapshot};

pub(super) fn cleanup_cache<R: Runtime>(app: &AppHandle<R>) {
    let Ok(directory) = app.path().app_cache_dir() else {
        return;
    };
    if let Err(error) = std::fs::remove_dir_all(directory.join("dictation")) {
        if error.kind() != std::io::ErrorKind::NotFound {
            log::warn!("failed to clean stale dictation audio: {error}");
        }
    }
}

pub(super) fn start<R: Runtime>(
    app: &AppHandle<R>,
    sender: &UnboundedSender<DictationAction>,
    snapshot: &Arc<RwLock<DictationSnapshot>>,
    active: &mut ActiveSession,
) {
    if !matches!(active, ActiveSession::Idle) {
        return;
    }
    let id = uuid::Uuid::new_v4().to_string();
    let capture = (|| {
        let target = insertion::capture_target()
            .map_err(|error| DictationFailure::classified("dictation_target_unavailable", error))?;
        let directory = app
            .path()
            .app_cache_dir()
            .map_err(|error| {
                DictationFailure::classified("dictation_audio_cache_failed", error.to_string())
            })?
            .join("dictation");
        std::fs::create_dir_all(&directory).map_err(|error| {
            DictationFailure::classified("dictation_audio_cache_failed", error.to_string())
        })?;
        let wav = directory.join(format!("{id}.wav"));
        let recorder = Recorder::start(&id, sender.clone())?;
        Ok::<_, DictationFailure>((target, wav, recorder))
    })();
    match capture {
        Ok((target, wav, recorder)) => {
            let target_name = target.display_name().to_owned();
            *active = ActiveSession::Recording {
                id: id.clone(),
                target,
                recorder,
                wav,
            };
            completion::publish(
                app,
                snapshot,
                DictationSnapshot {
                    phase: DictationPhase::Recording,
                    session_id: Some(id),
                    started_at: Some(epoch_millis()),
                    target_application: Some(target_name),
                    ..DictationSnapshot::default()
                },
            );
        }
        Err(error) => completion::fail(app, snapshot, Some(id), error),
    }
}

pub(super) async fn stop<R: Runtime>(
    app: &AppHandle<R>,
    sender: &UnboundedSender<DictationAction>,
    snapshot: &Arc<RwLock<DictationSnapshot>>,
    active: &mut ActiveSession,
) {
    let session = std::mem::replace(active, ActiveSession::Idle);
    let ActiveSession::Recording {
        id,
        target,
        recorder,
        wav,
    } = session
    else {
        *active = session;
        return;
    };
    let output = wav.clone();
    let recording = match crate::blocking::run(move || recorder.finish(&output)).await {
        Ok(value) => value,
        Err(error) => {
            let _ = tokio::fs::remove_file(&wav).await;
            completion::fail(app, snapshot, Some(id), error);
            return;
        }
    };
    let cancellation = CancellationToken::new();
    *active = ActiveSession::Transcribing {
        id: id.clone(),
        cancellation: cancellation.clone(),
    };
    completion::publish(
        app,
        snapshot,
        DictationSnapshot {
            phase: DictationPhase::PreparingResources,
            session_id: Some(id.clone()),
            target_application: Some(target.display_name().to_owned()),
            ..DictationSnapshot::default()
        },
    );
    spawn_transcription(
        app.clone(),
        sender.clone(),
        id,
        target,
        wav,
        recording,
        cancellation,
    );
}

fn spawn_transcription<R: Runtime>(
    app: AppHandle<R>,
    sender: UnboundedSender<DictationAction>,
    id: String,
    target: insertion::TargetApplication,
    wav: std::path::PathBuf,
    recording: RecordedAudio,
    cancellation: CancellationToken,
) {
    tauri::async_runtime::spawn(async move {
        let capture = transcription::transcribe(
            &app,
            &id,
            wav.clone(),
            recording,
            &cancellation,
            &mut |progress| {
                let _ = sender.send(DictationAction::PreparationProgress {
                    session_id: id.clone(),
                    progress,
                });
            },
        )
        .await;
        if let Err(error) = tokio::fs::remove_file(&wav).await {
            if error.kind() != std::io::ErrorKind::NotFound {
                log::warn!("failed to remove temporary dictation audio: {error}");
            }
        }
        let _ = sender.send(DictationAction::Finished {
            session_id: id,
            target,
            transcription: capture,
        });
    });
}

pub(super) fn cancel<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: &Arc<RwLock<DictationSnapshot>>,
    active: &mut ActiveSession,
) {
    match std::mem::replace(active, ActiveSession::Idle) {
        ActiveSession::Recording { wav, .. } => {
            let _ = std::fs::remove_file(wav);
        }
        ActiveSession::Transcribing { cancellation, .. } => cancellation.cancel(),
        ActiveSession::Idle => {}
    }
    completion::publish(app, snapshot, DictationSnapshot::default());
}

pub(super) fn device_failed<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: &Arc<RwLock<DictationSnapshot>>,
    active: &mut ActiveSession,
    session_id: String,
    failure: DictationFailure,
) {
    if matches!(active, ActiveSession::Recording { id, .. } if id == &session_id) {
        *active = ActiveSession::Idle;
        let (code, message) = failure.into_parts();
        completion::publish(
            app,
            snapshot,
            DictationSnapshot::failed(Some(session_id), code, message),
        );
        completion::reveal(app);
    }
}

fn epoch_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

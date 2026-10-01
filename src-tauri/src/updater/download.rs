use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

use super::contracts::{UpdatePhase, UpdateSnapshot};
use super::failure;
use super::state::{busy, require_version, DownloadOutcome, UpdateState};
use crate::app_error::{AppError, AppResult};

pub(super) async fn run(app: &AppHandle, version: &str) -> AppResult<UpdateSnapshot> {
    let state = app.state::<UpdateState>();
    let _operation = state.operation.try_lock().map_err(|_| busy())?;
    let token = CancellationToken::new();
    let mut update = {
        let session = state.lock()?;
        require_version(&session.snapshot, UpdatePhase::Available, version)?;
        session
            .pending
            .clone()
            .ok_or_else(|| AppError::internal("Missing selected update"))?
    };
    update.timeout = Some(Duration::from_secs(30 * 60));
    state.publish(|session| {
        session.cancellation = Some(token.clone());
        session.snapshot.phase = UpdatePhase::Downloading;
        session.snapshot.failure = None;
        session.snapshot.downloaded = 0;
        session.snapshot.total = None;
    })?;
    let mut downloaded = 0_u64;
    let mut last_progress = Instant::now();
    let download = update.download(
        |chunk, total| {
            downloaded += chunk as u64;
            if last_progress.elapsed() < Duration::from_millis(100) {
                return;
            }
            last_progress = Instant::now();
            if state
                .publish(|session| {
                    session.snapshot.downloaded = downloaded;
                    session.snapshot.total = total;
                })
                .is_err()
            {
                token.cancel();
            }
        },
        || {
            if state
                .publish(|session| session.snapshot.phase = UpdatePhase::Verifying)
                .is_err()
            {
                token.cancel();
            }
        },
    );
    let result = tokio::select! {
        biased;
        _ = token.cancelled() => return state.complete_download(DownloadOutcome::Cancelled),
        result = download => result,
    };
    match result {
        Ok(bytes) => state.complete_download(DownloadOutcome::Verified(bytes)),
        Err(error) => {
            let (kind, error) = failure::classify(error);
            let snapshot = state.complete_download(DownloadOutcome::Failed(kind))?;
            if snapshot.failure.is_some() {
                Err(error)
            } else {
                Ok(snapshot)
            }
        }
    }
}

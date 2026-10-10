use std::{
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::sync::{watch, Mutex};
use tokio_util::sync::CancellationToken;

pub(crate) mod admission;
mod artifact;
pub(crate) mod compute_budget;
pub(crate) mod device_profiles;
pub(crate) mod image;
mod process;
mod protocol;
mod runtime_profile;
pub(crate) mod separation;
pub(crate) mod speech;
mod supervisor;

static EXIT: OnceLock<CancellationToken> = OnceLock::new();
type WorkerSlot = Arc<Mutex<Option<process::Worker>>>;
static WORKER: OnceLock<WorkerSlot> = OnceLock::new();
static INFERENCE_ROOT_READY: OnceLock<watch::Receiver<Option<Result<(), String>>>> =
    OnceLock::new();
const IDLE_TTL: Duration = Duration::from_secs(60);

async fn prepare_inference_root(work_directory: &std::path::Path) -> Result<(), String> {
    let root = work_directory
        .parent()
        .ok_or("invalid_output: inference directory has no root")?
        .to_owned();
    let name = work_directory
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("invalid_output: inference session name is invalid")?;
    if uuid::Uuid::parse_str(name).is_err() {
        return Err("invalid_output: inference session name is not a UUID".into());
    }
    let mut receiver = INFERENCE_ROOT_READY
        .get_or_init(|| {
            let (sender, receiver) = watch::channel(None);
            tokio::task::spawn_blocking(move || {
                sender.send_replace(Some(clean_stale_inference_sessions(&root)));
            });
            receiver
        })
        .clone();
    loop {
        if let Some(cleanup_outcome) = receiver.borrow_and_update().clone() {
            return cleanup_outcome;
        }
        receiver
            .changed()
            .await
            .map_err(|_| "invalid_output: inference cleanup stopped".to_string())?;
    }
}

fn clean_stale_inference_sessions(root: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(root).map_err(|error| error.to_string())?;
    let metadata = std::fs::symlink_metadata(root).map_err(|error| error.to_string())?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || artifact::is_reparse_point(&metadata)
    {
        return Err("invalid_output: inference root is not a private directory".into());
    }
    for entry in std::fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or("invalid_output: stale inference entry has invalid name")?;
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if uuid::Uuid::parse_str(name).is_err()
            || !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || artifact::is_reparse_point(&metadata)
        {
            return Err("invalid_output: unexpected entry in inference cache".into());
        }
        std::fs::remove_dir_all(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(super) fn exit_token() -> &'static CancellationToken {
    EXIT.get_or_init(CancellationToken::new)
}

fn worker_slot() -> WorkerSlot {
    WORKER.get_or_init(|| Arc::new(Mutex::new(None))).clone()
}

async fn prepare_worker_for_admission<F>(
    priority: admission::Priority,
    compatible: F,
) -> Result<(), String>
where
    F: FnOnce(&process::Worker) -> bool,
{
    let slot = worker_slot();
    let Ok(mut guard) = slot.try_lock() else {
        return Ok(());
    };
    let Some(worker) = guard.as_mut() else {
        return Ok(());
    };
    let resident_bytes = match worker.resident_bytes().await {
        Ok(bytes) => bytes,
        Err(error) => {
            worker.terminate().await;
            *guard = None;
            return Err(error);
        }
    };
    let reclaim_for_memory = admission::memory_pressure()?;
    let reclaim_for_priority = priority == admission::Priority::Interactive && !compatible(worker);
    if reclaim_for_memory || reclaim_for_priority {
        log::info!("reclaiming idle inference worker resident_bytes={resident_bytes}");
        worker.terminate().await;
        *guard = None;
    }
    Ok(())
}

pub(super) fn deadline_duration(deadline_ms: u64) -> Result<Duration, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis();
    let remaining = u128::from(deadline_ms)
        .checked_sub(now)
        .ok_or("deadline_exceeded")?;
    Ok(Duration::from_millis(
        remaining.min(u128::from(u64::MAX)) as u64
    ))
}

fn reclaim_when_idle(slot: WorkerSlot) {
    tokio::spawn(async move {
        tokio::time::sleep(IDLE_TTL).await;
        let mut guard = slot.lock().await;
        if guard
            .as_ref()
            .is_some_and(|worker| worker.last_used.elapsed() >= IDLE_TTL)
        {
            if let Some(worker) = guard.as_mut() {
                let _ = tokio::time::timeout(
                    Duration::from_secs(2),
                    worker
                        .protocol
                        .request::<_, crate::ai::contracts::runtime::RuntimeShutdownAcknowledgement>(
                            &worker.session_id.clone(),
                            "runtime.shutdown",
                            &crate::ai::contracts::runtime::RuntimeShutdownParams {
                                session_id: worker.session_id.clone(),
                            },
                        ),
                )
                .await;
                worker.terminate().await;
            }
            *guard = None;
        }
    });
}

pub(crate) use image::segment_image;
pub(crate) use supervisor::encode_embedding;

pub(crate) async fn shutdown() {
    exit_token().cancel();
    supervisor::shutdown().await;
}

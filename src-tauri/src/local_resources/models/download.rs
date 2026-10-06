use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use tauri::{AppHandle, Runtime};
use tokio::sync::{mpsc, Semaphore};
use tokio_util::sync::CancellationToken;

use super::{
    contracts::{DownloadEvent, DownloadRequest},
    snapshots::SnapshotKind,
    sources,
};
use crate::local_resources::{
    preferences::ResourceDownloadRegion,
    preparation::{
        contracts::{ResourcePreparationProgress, ResourcePreparationStage},
        RESOURCE_PREPARATION_CANCELLED,
    },
    runtimes::python,
};

static DOWNLOADS: Semaphore = Semaphore::const_new(2);

pub(super) async fn snapshot<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: SnapshotKind,
    region: ResourceDownloadRegion,
    directory: &Path,
    progress: &mut impl FnMut(ResourcePreparationProgress),
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let _permit = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(RESOURCE_PREPARATION_CANCELLED.into()),
        permit = DOWNLOADS.acquire() => permit.map_err(|error| error.to_string())?,
    };
    let runtime = python::resolve(app, python::RuntimeProfile::Inference)?
        .ok_or("Python runtime is not prepared")?;
    let request = DownloadRequest {
        directory,
        source: sources::for_snapshot(snapshot, region),
        files: snapshot.files(),
    };
    let payload = serde_json::to_vec(&request).map_err(|error| error.to_string())?;
    let cancel = cancellation.clone();
    let (updates, mut receiver) = mpsc::channel(16);
    let cache = directory
        .parent()
        .and_then(Path::parent)
        .ok_or("Model resource root is unavailable")?
        .join("hub-cache");
    let mut run = tokio::task::spawn_blocking(move || {
        run_download(
            runtime,
            payload,
            cache,
            snapshot.total_bytes(),
            cancel,
            updates,
        )
    });
    loop {
        tokio::select! {
            update = receiver.recv() => {
                if let Some(update) = update { progress(update); }
                else { return (&mut run).await.map_err(|error| error.to_string())?; }
            }
            outcome = &mut run => return outcome.map_err(|error| error.to_string())?,
        }
    }
}

fn run_download(
    runtime: python::PythonRuntimePaths,
    payload: Vec<u8>,
    cache: std::path::PathBuf,
    expected_total: u64,
    cancellation: CancellationToken,
    updates: mpsc::Sender<ResourcePreparationProgress>,
) -> Result<(), String> {
    python::verify_before_spawn(&runtime)?;
    if cancellation.is_cancelled() {
        return Err(RESOURCE_PREPARATION_CANCELLED.into());
    }
    let bootstrap = "import sys; sys.path[:0] = [sys.argv[1], sys.argv[2]]; from knovum_model_resources.server import main; main()";
    let mut command = Command::new(&runtime.executable);
    command
        .args(["-I", "-B", "-X", "utf8", "-c", bootstrap])
        .arg(&runtime.worker_root)
        .arg(&runtime.package_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("HF_HUB_DISABLE_TELEMETRY", "1")
        .env("HF_HUB_DISABLE_IMPLICIT_TOKEN", "1")
        .env("MODELSCOPE_API_TOKEN", "")
        .env("MODELSCOPE_ENDPOINT", "https://modelscope.cn")
        .env("HF_HOME", cache.join("hugging-face"))
        .env("HF_HUB_CACHE", cache.join("hugging-face/hub"))
        .env("HF_XET_CACHE", cache.join("hugging-face/xet"))
        .env("MODELSCOPE_HOME", cache.join("model-scope"))
        .env("MODELSCOPE_CACHE", cache.join("model-scope/hub"));
    for key in [
        "PYTHONPATH",
        "PYTHONHOME",
        "PYTHONSTARTUP",
        "HF_TOKEN",
        "HF_TOKEN_PATH",
        "HUGGING_FACE_HUB_TOKEN",
        "HF_ENDPOINT",
        "HF_HUB_OFFLINE",
        "MODELSCOPE_DOMAIN",
    ] {
        command.env_remove(key);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let write = child
        .stdin
        .take()
        .ok_or_else(|| "Model downloader stdin is unavailable".to_owned())
        .and_then(|mut input| {
            input
                .write_all(&payload)
                .and_then(|_| input.write_all(b"\n"))
                .map_err(|error| error.to_string())
        });
    if let Err(error) = write {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    let mut complete = false;
    let outcome = crate::subprocess::wait_with_output(
        child,
        &cancellation,
        RESOURCE_PREPARATION_CANCELLED,
        |line| observe(line, expected_total, &mut complete, &updates),
        |_| Ok(()),
    )?;
    if !outcome.status.success() || !complete {
        return Err("Model downloader ended without successful completion".into());
    }
    Ok(())
}

fn observe(
    line: &str,
    expected_total: u64,
    complete: &mut bool,
    updates: &mpsc::Sender<ResourcePreparationProgress>,
) -> Result<(), String> {
    if *complete {
        return Err("Model downloader emitted output after completion".into());
    }
    match serde_json::from_str(line)
        .map_err(|error| format!("Invalid model download event: {error}"))?
    {
        DownloadEvent::Progress { downloaded, total } => {
            if total != expected_total || downloaded > total {
                return Err("Model download progress differs from the requested snapshot".into());
            }
            let update = ResourcePreparationProgress {
                stage: ResourcePreparationStage::Downloading,
                downloaded,
                total: Some(total),
                percent: Some(((downloaded.saturating_mul(100) / total).min(100)) as u8),
            };
            let _ = updates.try_send(update);
        }
        DownloadEvent::Complete {} => *complete = true,
        DownloadEvent::Error { message } => return Err(message),
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/download.rs"]
mod tests;

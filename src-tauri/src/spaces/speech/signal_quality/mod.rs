mod scan;
#[cfg(test)]
mod tests;

use std::{
    path::Path,
    process::{Command, Stdio},
};

use serde::Serialize;
use tauri::{ipc::Channel, AppHandle};

use super::{cancellation, publication::hash_file};
use crate::{
    foundation::app_error::AppError,
    foundation::subprocess,
    local_resources::tools,
    spaces::{assets, db},
};

#[derive(Clone, Copy, Serialize, PartialEq, Eq, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum SignalWarningKind {
    Empty,
    Silence,
    Quiet,
    Loud,
    Clipping,
    Invalid,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SignalWarning {
    pub kind: SignalWarningKind,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechSignalReport {
    pub scanned_duration_ms: u64,
    pub truncated: bool,
    pub warnings: Vec<SignalWarning>,
}

#[tauri::command]
pub async fn inspect_speech_audio(
    app: AppHandle,
    project_id: String,
    revision_id: String,
    job_id: String,
    on_progress: Channel<u64>,
) -> Result<SpeechSignalReport, AppError> {
    let (cancellation, _lease) = cancellation::acquire(&job_id)?;
    let (source, sha256): (String, String) = sqlx::query_as("SELECT r.output_asset_path,r.output_sha256 FROM spaces_speech_project_revisions r JOIN spaces_speech_projects p ON p.id=r.project_id WHERE p.id=? AND r.id=? AND r.output_asset_path IS NOT NULL")
        .bind(project_id).bind(revision_id).fetch_optional(db::pool(&app).await?).await.map_err(db::sql_error)?
        .ok_or("Generate a complete project output before checking its audio")?;
    let source = assets::authorize_spaces_asset(&app, Path::new(&source))?;
    let ffmpeg = tools::resolve_ffmpeg(&app)?.ok_or("Bundled FFmpeg is unavailable")?;
    Ok(
        crate::foundation::blocking::run(move || -> Result<_, String> {
            if hash_file(&source)? != sha256 {
                return Err("Speech output changed before signal inspection".into());
            }
            let report = inspect_file(&ffmpeg, &source, &cancellation, |milliseconds| {
                let _ = on_progress.send(milliseconds);
            })?;
            if hash_file(&source)? != sha256 {
                return Err("Speech output changed during signal inspection".into());
            }
            Ok(report)
        })
        .await?,
    )
}

fn inspect_file(
    ffmpeg: &Path,
    source: &Path,
    cancellation: &tokio_util::sync::CancellationToken,
    mut progress: impl FnMut(u64),
) -> Result<SpeechSignalReport, String> {
    let filter = "aresample=48000,aformat=sample_fmts=dblp,asetpts=N/SR/TB,asetnsamples=n=4800:p=0,astats=metadata=1:reset=1:measure_perchannel=Number_of_NaNs+Number_of_Infs:measure_overall=Peak_level+RMS_level+Number_of_samples,ametadata=mode=print:file=-";
    let child = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-i"])
        .arg(source)
        .args([
            "-t", "7200.1", "-map", "0:a:0", "-af", filter, "-f", "null", "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let mut scanner = scan::WindowScanner::new();
    let mut reported_windows = 0;
    let outcome = subprocess::wait_with_output(
        child,
        cancellation,
        "cancelled",
        |line| {
            scanner.read(line)?;
            if scanner.window_count() >= reported_windows + 100 {
                reported_windows = scanner.window_count();
                progress(scanner.scanned_ms());
            }
            Ok(())
        },
        |_| Ok(()),
    )?;
    if !outcome.status.success() {
        return Err(subprocess::diagnostic_error(
            &outcome.diagnostics,
            "Audio signal inspection failed",
        ));
    }
    scanner.finish()
}

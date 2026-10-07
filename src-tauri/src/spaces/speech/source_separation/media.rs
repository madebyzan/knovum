use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};
use tauri::Manager;
use tokio_util::sync::CancellationToken;

pub(super) const SAMPLE_RATE: u32 = 44_100;
pub(super) const MAX_SAMPLE_COUNT: u64 = SAMPLE_RATE as u64 * 10 * 60;

pub(super) fn extract_audio(
    ffmpeg: &Path,
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let child = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"])
        .arg(source)
        .args([
            "-progress",
            "pipe:1",
            "-stats_period",
            "0.25",
            "-nostats",
            "-fflags",
            "+bitexact",
            "-flags:a",
            "+bitexact",
            "-map_metadata",
            "-1",
            "-map",
            "0:a:0",
            "-vn",
            "-ar",
            "44100",
            "-ac",
            "2",
            "-c:a",
            "pcm_s16le",
            "-f",
            "wav",
        ])
        .arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to launch FFmpeg: {error}"))?;
    let max_bytes = MAX_SAMPLE_COUNT * 4 + 44;
    let output = crate::subprocess::wait_with_output(
        child,
        cancellation,
        "Source separation was cancelled",
        |_| match target.metadata() {
            Ok(metadata) if metadata.len() > max_bytes => Err(
                "Source separation requires decoded audio lasting no more than ten minutes".into(),
            ),
            Ok(_) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!(
                "Source separation output cannot be inspected: {error}"
            )),
        },
        |_| Ok(()),
    )?;
    if !output.status.success() {
        return Err(crate::subprocess::diagnostic_error(
            &output.diagnostics,
            "FFmpeg could not decode the video audio track",
        ));
    }
    Ok(())
}

pub(super) fn workspace(app: &tauri::AppHandle, attempt_id: &str) -> Result<PathBuf, String> {
    let parsed = uuid::Uuid::parse_str(attempt_id)
        .map_err(|_| "Source separation attempt identity is invalid")?;
    if parsed.to_string() != attempt_id {
        return Err("Source separation attempt identity is not canonical".into());
    }
    let path = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("speech-source-separation")
        .join(attempt_id);
    fs::create_dir_all(
        path.parent()
            .ok_or("Source separation cache root is unavailable")?,
    )
    .map_err(|error| error.to_string())?;
    fs::create_dir(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

pub(super) fn pcm_wav_metadata(path: &Path) -> Result<(u64, u64), String> {
    let metadata = crate::media::wav::metadata(path)?;
    let expected = super::super::audio_format::WavPrecision::Pcm16.spec(SAMPLE_RATE, 2);
    if metadata.spec != expected {
        return Err("FFmpeg source separation audio is not 44.1 kHz stereo PCM16 WAV".into());
    }
    let sample_count = metadata.sample_count;
    if sample_count == 0 || sample_count > MAX_SAMPLE_COUNT {
        return Err(
            "Source separation requires decoded audio lasting no more than ten minutes".into(),
        );
    }
    Ok((metadata.byte_length, sample_count))
}

pub(super) fn deadline(duration_ms: u64) -> Result<u64, String> {
    let allowance_ms = duration_ms
        .checked_mul(3)
        .map(|value| value.max(10 * 60 * 1_000))
        .ok_or("Source separation deadline overflow")?;
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis(),
    )
    .map_err(|_| "System time exceeds protocol range".to_string())?
    .checked_add(allowance_ms)
    .ok_or("Source separation deadline overflow".into())
}

pub(super) fn hash_file(
    path: &Path,
    cancellation: &CancellationToken,
    deadline_ms: u64,
) -> Result<String, String> {
    let mut source = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        if cancellation.is_cancelled() {
            return Err("Source separation was cancelled".into());
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis();
        if now >= u128::from(deadline_ms) {
            return Err("deadline_exceeded".into());
        }
        let read = source
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Ok(hex::encode(digest.finalize()));
        }
        digest.update(&buffer[..read]);
    }
}

pub(in crate::spaces::speech) fn cleanup_stale_workspaces(
    app: &tauri::AppHandle,
) -> Result<(), String> {
    let root = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("speech-source-separation");
    if !root.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(&root).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Source separation cache root is invalid".into());
    }
    for entry in fs::read_dir(&root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let name = entry
            .file_name()
            .to_str()
            .ok_or("Source separation cache entry has an invalid name")?
            .to_owned();
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if uuid::Uuid::parse_str(&name)
            .map(|id| id.to_string() != name)
            .unwrap_or(true)
            || !metadata.is_dir()
            || metadata.file_type().is_symlink()
        {
            return Err("Source separation cache contains an unexpected entry".into());
        }
        fs::remove_dir_all(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

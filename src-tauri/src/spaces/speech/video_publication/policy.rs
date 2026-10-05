use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::super::contracts::SpeechSegmentRenderOutput;
use crate::{file_inspection::verify_content_revision, spaces::assets};

pub(in crate::spaces::speech) fn verified_source<R: Runtime>(
    app: &AppHandle<R>,
    path: &str,
    revision: &str,
) -> Result<PathBuf, String> {
    let path = assets::authorize_spaces_asset(app, Path::new(path))?;
    verify_content_revision(&path, revision)?;
    Ok(path)
}

pub(super) fn assembly_digest(
    renders: &[SpeechSegmentRenderOutput],
    windows: &[(Option<i64>, Option<i64>)],
    config: &serde_json::Value,
) -> String {
    let mut digest = Sha256::new();
    for (render, window) in renders.iter().zip(windows) {
        digest.update(render.segment_id.as_bytes());
        digest.update(render.sha256.as_bytes());
        digest.update(window.0.unwrap_or(-1).to_le_bytes());
        digest.update(window.1.unwrap_or(-1).to_le_bytes());
    }
    digest.update(config.to_string().as_bytes());
    format!("{:x}", digest.finalize())
}

pub(super) fn wav_sample_count(path: &Path, channels: u16) -> Result<u64, String> {
    let metadata = crate::media::wav::metadata(path)?;
    let expected =
        super::super::audio_format::WavPrecision::Float32.spec(super::SAMPLE_RATE, channels);
    if metadata.spec != expected {
        return Err("Video dubbing WAV is not float32 with the declared channels at 48 kHz".into());
    }
    Ok(metadata.sample_count)
}

pub(in crate::spaces::speech) fn verified_duration_us(
    ffmpeg: &Path,
    source: &Path,
    stream: &str,
    expected_ms: u64,
    cancellation: &CancellationToken,
) -> Result<u64, String> {
    let actual_us = crate::media::ffmpeg::duration_us(ffmpeg, source, stream, cancellation)?;
    let expected_us = expected_ms
        .checked_mul(1_000)
        .ok_or("Media duration overflow")?;
    if actual_us.abs_diff(expected_us) > 250_000 {
        return Err("Media duration differs from the analyzed source metadata".into());
    }
    Ok(actual_us)
}

pub(in crate::spaces::speech) fn samples_for_duration(duration_us: u64) -> Result<u64, String> {
    let samples = u128::from(duration_us)
        .checked_mul(u128::from(super::SAMPLE_RATE))
        .ok_or("Video duration sample count overflow")?
        .div_ceil(1_000_000);
    u64::try_from(samples).map_err(|_| "Video duration sample count exceeds platform range".into())
}
